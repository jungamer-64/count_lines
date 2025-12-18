use crate::config::{FilterConfig, WalkOptions};
use crate::error::Result;
use crate::path_security::{PathSanitizeOptions, is_path_safe, sanitize_path};
use crossbeam_channel::Sender;
use ignore::WalkBuilder;
use std::path::PathBuf;

/// Parallel recursive directory walk.
///
/// Validates root paths before walking for security.
///
/// # Errors
/// Returns `Ok` if traversal completes. Errors during traversal are handled internally or ignored.
/// Returns an error if any root path fails security validation.
pub fn walk_parallel(
    options: &WalkOptions,
    filters: &FilterConfig,
    tx: &Sender<(PathBuf, std::fs::Metadata)>,
) -> Result<()> {
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
            return Err(crate::error::AppError::Config(format!(
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

    for ov in &options.override_include {
        let mut ov_builder = ignore::overrides::OverrideBuilder::new(&options.roots[0]);
        if let Ok(o) = ov_builder.add(ov).and_then(|b| b.build()) {
            builder.overrides(o);
        }
    }
    // TODO: handle override_exclude properly (ignore crate uses !pattern for exclude in add method usually?)
    // ignore::overrides::OverrideBuilder add method documentation:
    // "A glob pattern that matches a path will be whitelisted."
    // "A glob pattern starting with ! that matches a path will be ignored."

    if let Some(types) = &options.types {
        builder.types(types.clone());
    }

    let filters = filters.clone();
    builder.filter_entry(move |entry| {
        // Always descend into directories (unless max_depth handles it, which it does)
        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            return true;
        }

        // Extension filter
        if !filters.allow_ext.is_empty() {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if !filters.allow_ext.iter().any(|e| e == ext) {
                    return false;
                }
            } else {
                // No extension: if whitelist exists, skip file
                return false;
            }
        }

        // Size and mtime filters
        // Note: metadata() might trigger stat if not cached.
        // ignore crate usually stats.
        if (filters.min_size.is_some()
            || filters.max_size.is_some()
            || filters.mtime_since.is_some()
            || filters.mtime_until.is_some())
            && let Ok(meta) = entry.metadata()
        {
            let size = meta.len();
            if let Some(min) = filters.min_size
                && size < min
            {
                return false;
            }
            if let Some(max) = filters.max_size
                && size > max
            {
                return false;
            }

            // Mtime filter
            if (filters.mtime_since.is_some() || filters.mtime_until.is_some())
                && let Ok(mod_time) = meta.modified()
            {
                let dt: chrono::DateTime<chrono::Local> = mod_time.into();
                if let Some(since) = filters.mtime_since
                    && dt < since
                {
                    return false;
                }
                if let Some(until) = filters.mtime_until
                    && dt > until
                {
                    return false;
                }
            }
        }

        // Default include
        true
    });

    let walker = builder.build_parallel();
    walker.run(|| {
        let tx = tx.clone();
        Box::new(move |entry| {
            if let Ok(entry) = entry
                && entry.file_type().is_some_and(|ft| ft.is_file())
            {
                // Try to get metadata. If it fails, we might just skip or log?
                // But usually if we found the entry, we can get metadata (unless it vanished).
                if let Ok(meta) = entry.metadata() {
                    let _ = tx.send((entry.path().to_owned(), meta));
                }
            }
            ignore::WalkState::Continue
        })
    });

    Ok(())
}
