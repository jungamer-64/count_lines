use crate::config::{FilterConfig, WalkOptions};
use crate::error::Result;
use crate::path_security::{PathSanitizeOptions, is_path_safe, sanitize_path};
use ignore::WalkBuilder;

/// Parallel recursive directory walk.
///
/// Validates root paths before walking for security.
///
/// # Errors
/// Returns `Ok` if traversal completes. Errors during traversal are handled internally or ignored.
/// Returns an error if any root path fails security validation.
pub fn walk_parallel<F>(
    options: &WalkOptions,
    _filters: &FilterConfig,
    processor: F,
) -> Result<()>
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
    if !options.override_include.is_empty() || !options.override_exclude.is_empty() {
        let mut ov_builder = ignore::overrides::OverrideBuilder::new(&options.roots[0]);
        for ov in &options.override_include {
            let _ = ov_builder.add(ov);
        }
        for ov in &options.override_exclude {
            let _ = ov_builder.add(&format!("!{ov}"));
        }
        if let Ok(overrides) = ov_builder.build() {
            builder.overrides(overrides);
        }
    }

    if let Some(types) = &options.types {
        builder.types(types.clone());
    }

    // Extension filtering is now handled by builder.types() if provided in WalkOptions

    let processor = std::sync::Arc::new(processor);
    let walker = builder.build_parallel();
    walker.run(|| {
        let processor = processor.clone();
        Box::new(move |entry| {
            if let Ok(entry) = entry
                && entry.file_type().is_some_and(|ft| ft.is_file())
            {
                if let Ok(meta) = entry.metadata() {
                    processor(entry.path().to_owned(), meta);
                }
            }
            ignore::WalkState::Continue
        })
    });

    Ok(())
}
