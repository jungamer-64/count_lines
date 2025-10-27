use crate::domain::config::{Config, Filters};
use chrono::{DateTime, Local};
use std::path::Path;

/// Default directories pruned unless `no_default_prune` is set.
const DEFAULT_PRUNE_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".venv",
    "venv",
    "build",
    "dist",
    "target",
    ".cache",
    ".direnv",
    ".mypy_cache",
    ".pytest_cache",
    "coverage",
    "__pycache__",
    ".idea",
    ".next",
    ".nuxt",
];

pub(super) fn should_process_entry(entry: &walkdir::DirEntry, config: &Config) -> bool {
    let path = entry.path();
    if !config.hidden && is_hidden(path) {
        return false;
    }
    if !config.no_default_prune && entry.file_type().is_dir() {
        let name = entry.file_name().to_string_lossy();
        if DEFAULT_PRUNE_DIRS.contains(&name.as_ref()) {
            return false;
        }
    }
    if entry.file_type().is_dir() {
        return !config
            .filters
            .exclude_dirs
            .iter()
            .any(|p| p.matches_path(path));
    }
    true
}

pub(super) struct PathMatcher;

impl PathMatcher {
    pub(super) fn matches(path: &Path, config: &Config) -> bool {
        let filters = &config.filters;
        matches_name(path, filters)
            && matches_path_patterns(path, filters)
            && matches_extension(path, filters)
            && matches_metadata(path, config)
    }
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .map_or(false, |name| name.to_string_lossy().starts_with('.'))
}

fn matches_name(path: &Path, filters: &Filters) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if !filters.include_patterns.is_empty()
        && !filters.include_patterns.iter().any(|p| p.matches(&name))
    {
        return false;
    }
    !filters.exclude_patterns.iter().any(|p| p.matches(&name))
}

fn matches_path_patterns(path: &Path, filters: &Filters) -> bool {
    if !filters.include_paths.is_empty()
        && !filters.include_paths.iter().any(|p| p.matches_path(path))
    {
        return false;
    }
    !filters.exclude_paths.iter().any(|p| p.matches_path(path))
}

fn matches_extension(path: &Path, filters: &Filters) -> bool {
    if filters.ext_filters.is_empty() {
        return true;
    }
    path.extension()
        .and_then(|e| Some(e.to_string_lossy().to_lowercase()))
        .map_or(false, |ext| filters.ext_filters.contains(&ext))
}

fn matches_metadata(path: &Path, config: &Config) -> bool {
    let filters = &config.filters;
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return true,
    };
    if !filters.size_range.contains(metadata.len()) {
        return false;
    }
    matches_mtime(&metadata, config)
}

fn matches_mtime(metadata: &std::fs::Metadata, config: &Config) -> bool {
    let Ok(modified_sys) = metadata.modified() else {
        return true;
    };
    let modified: DateTime<Local> = modified_sys.into();
    if let Some(since) = config.mtime_since {
        if modified < since {
            return false;
        }
    }
    if let Some(until) = config.mtime_until {
        if modified > until {
            return false;
        }
    }
    true
}
