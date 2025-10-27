mod git;
mod inputs;
mod matcher;
mod metadata;

use crate::domain::config::Config;
use crate::foundation::types::{FileEntry, FileMeta};
use std::path::PathBuf;
use walkdir::WalkDir;

/// Collect the list of file entries to be processed based on the provided configuration.
pub fn collect_entries(config: &Config) -> anyhow::Result<Vec<FileEntry>> {
    if let Some(ref from0) = config.files_from0 {
        return inputs::read_files_from_null(from0).map(|files| to_entries(files, config));
    }
    if let Some(ref from) = config.files_from {
        return inputs::read_files_from_lines(from).map(|files| to_entries(files, config));
    }
    if config.use_git {
        if let Ok(files) = git::collect_git_files(config) {
            return Ok(to_entries(files, config));
        }
    }
    collect_walk_entries(config)
}

fn to_entries(files: Vec<PathBuf>, config: &Config) -> Vec<FileEntry> {
    files
        .into_iter()
        .filter_map(|p| FileMeta::from_path(&p, config).map(|meta| FileEntry { path: p, meta }))
        .collect()
}

/// Walk the filesystem and collect entries matching the configured filters.
pub fn collect_walk_entries(config: &Config) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    for root in &config.paths {
        let walker = WalkDir::new(root)
            .follow_links(config.follow)
            .into_iter()
            .filter_entry(|e| matcher::should_process_entry(e, config));

        for entry in walker.flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if !matcher::PathMatcher::matches(path, config) {
                continue;
            }
            if let Some(meta) = FileMeta::from_path(path, config) {
                entries.push(FileEntry {
                    path: path.to_path_buf(),
                    meta,
                });
            }
        }
    }
    Ok(entries)
}
