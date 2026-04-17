use crate::config::{FilterConfig, WalkOptions};
use crate::error::{EngineError, Result};
use crate::path_security::{PathSanitizeOptions, is_path_safe, sanitize_path};
use hashbrown::HashSet;
use ignore::WalkBuilder;
use std::path::Path;

/// Parallel recursive directory walk.
///
/// Validates root paths before walking for security.
///
/// # Errors
/// Returns `Ok` if traversal completes. Errors during traversal are handled internally or ignored.
/// Returns an error if any root path fails security validation.
pub fn walk_parallel<F>(options: &WalkOptions, filters: &FilterConfig, processor: F) -> Result<()>
where
    F: Fn(std::path::PathBuf, std::fs::Metadata) + Send + Sync + 'static,
{
    if options.roots.is_empty() {
        return Ok(());
    }

    // Validate root paths for security
    let sanitize_opts = PathSanitizeOptions {
        allow_symlinks: options.follow_links,
        max_depth: options.max_depth.unwrap_or(256),
        ..Default::default()
    };

    for root in &options.roots {
        // Quick safety check (lightweight, no filesystem access)
        if !is_path_safe(root) {
            return Err(crate::error::EngineError::Config(format!(
                "Potentially unsafe path: {}",
                root.display()
            )));
        }

        // Full validation for existing paths
        if root.exists() {
            sanitize_path(root, &sanitize_opts)?;
        }
    }

    let mut builder = WalkBuilder::new(&options.roots[0]);
    for root in &options.roots[1..] {
        builder.add(root);
    }

    builder
        .threads(options.threads)
        .hidden(!options.hidden)
        .git_ignore(options.git_ignore)
        .follow_links(options.follow_links);

    if let Some(depth) = options.max_depth {
        builder.max_depth(Some(depth));
    }

    // Build overrides (include + exclude) in a single OverrideBuilder
    // ignore crate only supports one Overrides instance per WalkBuilder.
    // Exclude patterns use the `!` prefix convention.
    if !options.override_include.is_empty()
        || !options.override_exclude.is_empty()
        || !filters.include_patterns.is_empty()
        || !filters.exclude_patterns.is_empty()
    {
        let mut ov_builder = ignore::overrides::OverrideBuilder::new(&options.roots[0]);

        for ov in &options.override_include {
            ov_builder.add(ov).map_err(|err| {
                EngineError::Config(format!("Invalid override include pattern '{ov}': {err}"))
            })?;
        }

        for ov in &options.override_exclude {
            let pattern = format!("!{ov}");
            ov_builder.add(&pattern).map_err(|err| {
                EngineError::Config(format!("Invalid override exclude pattern '{ov}': {err}"))
            })?;
        }

        for pattern in &filters.include_patterns {
            ov_builder.add(pattern).map_err(|err| {
                EngineError::Config(format!("Invalid filter include pattern '{pattern}': {err}"))
            })?;
        }

        for pattern in &filters.exclude_patterns {
            let exclusion = format!("!{pattern}");
            ov_builder.add(&exclusion).map_err(|err| {
                EngineError::Config(format!("Invalid filter exclude pattern '{pattern}': {err}"))
            })?;
        }

        let overrides = ov_builder
            .build()
            .map_err(|err| EngineError::Config(format!("Failed to build overrides: {err}")))?;
        builder.overrides(overrides);
    }

    if let Some(types) = &options.types {
        builder.types(types.clone());
    }

    let allow_ext = collect_normalized_exts(&filters.allow_ext);
    let deny_ext = collect_normalized_exts(&filters.deny_ext);

    let processor = std::sync::Arc::new(processor);
    let walker = builder.build_parallel();
    walker.run(|| {
        let processor = processor.clone();
        let allow_ext = allow_ext.clone();
        let deny_ext = deny_ext.clone();
        let filters = filters.clone();

        Box::new(move |entry| {
            if let Ok(entry) = entry
                && entry.file_type().is_some_and(|ft| ft.is_file())
                && let Ok(meta) = entry.metadata()
            {
                let path = entry.path();
                if matches_filter(path, &meta, &filters, &allow_ext, &deny_ext) {
                    processor(path.to_owned(), meta);
                }
            }
            ignore::WalkState::Continue
        })
    });

    Ok(())
}

fn collect_normalized_exts(exts: &[String]) -> HashSet<String> {
    exts.iter()
        .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
        .collect()
}

fn extension_of(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim().trim_start_matches('.').to_ascii_lowercase())
        .filter(|ext| !ext.is_empty())
}

fn matches_filter(
    path: &Path,
    metadata: &std::fs::Metadata,
    filters: &FilterConfig,
    allow_ext: &HashSet<String>,
    deny_ext: &HashSet<String>,
) -> bool {
    let ext = extension_of(path);

    if !allow_ext.is_empty() && ext.as_ref().is_none_or(|value| !allow_ext.contains(value)) {
        return false;
    }

    if ext.as_ref().is_some_and(|value| deny_ext.contains(value)) {
        return false;
    }

    let size = metadata.len();
    if filters.min_size.is_some_and(|min| size < min) {
        return false;
    }
    if filters.max_size.is_some_and(|max| size > max) {
        return false;
    }

    if filters.mtime_since.is_some() || filters.mtime_until.is_some() {
        let Ok(modified) = metadata.modified() else {
            return false;
        };

        let modified = chrono::DateTime::<chrono::Local>::from(modified);
        if filters.mtime_since.is_some_and(|since| modified < since) {
            return false;
        }
        if filters.mtime_until.is_some_and(|until| modified > until) {
            return false;
        }
    }

    true
}
