use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, Local};

use crate::{
    config::{ByKey, Filters},
    options::{OutputFormat, SortKey, WatchOutput},
    value_objects::FileMeta,
};

/// Domain representation of resolved configuration options.
#[derive(Debug, Clone)]
pub struct Config {
    pub format: OutputFormat,
    pub sort_specs: Vec<(SortKey, bool)>,
    pub top_n: Option<usize>,
    pub by_modes: Vec<ByKey>,
    pub summary_only: bool,
    pub total_only: bool,
    pub by_limit: Option<usize>,
    pub filters: Filters,
    pub hidden: bool,
    pub follow: bool,
    pub use_git: bool,
    pub jobs: usize,
    pub no_default_prune: bool,
    pub abs_path: bool,
    pub abs_canonical: bool,
    pub trim_root: Option<PathBuf>,
    pub words: bool,
    pub count_newlines_in_chars: bool,
    pub text_only: bool,
    pub fast_text_detect: bool,
    pub files_from: Option<PathBuf>,
    pub files_from0: Option<PathBuf>,
    pub paths: Vec<PathBuf>,
    pub mtime_since: Option<DateTime<Local>>,
    pub mtime_until: Option<DateTime<Local>>,
    pub total_row: bool,
    pub progress: bool,
    pub ratio: bool,
    pub output: Option<PathBuf>,
    pub strict: bool,
    pub incremental: bool,
    pub cache_dir: Option<PathBuf>,
    pub cache_verify: bool,
    pub clear_cache: bool,
    pub watch: bool,
    pub watch_interval: Duration,
    pub watch_output: WatchOutput,
    pub compare: Option<(PathBuf, PathBuf)>,
}

impl Config {
    /// Determine whether a file at `path` with the given metadata should be included.
    pub fn matches_file(&self, path: &Path, meta: &FileMeta) -> bool {
        self.matches_name_patterns(path)
            && self.matches_path_patterns(path)
            && self.matches_extension(meta)
            && self.matches_metadata(meta)
    }

    fn matches_name_patterns(&self, path: &Path) -> bool {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };

        let filters = &self.filters;

        if !filters.include_patterns.is_empty()
            && !filters.include_patterns.iter().any(|pattern| pattern.matches(file_name))
        {
            return false;
        }

        !filters.exclude_patterns.iter().any(|pattern| pattern.matches(file_name))
    }

    fn matches_path_patterns(&self, path: &Path) -> bool {
        let filters = &self.filters;

        if !filters.include_paths.is_empty()
            && !filters.include_paths.iter().any(|pattern| pattern.matches_path(path))
        {
            return false;
        }

        !filters.exclude_paths.iter().any(|pattern| pattern.matches_path(path))
    }

    fn matches_extension(&self, meta: &FileMeta) -> bool {
        let filters = &self.filters;
        if filters.ext_filters.is_empty() {
            return true;
        }

        filters.ext_filters.contains(&meta.ext)
    }

    fn matches_metadata(&self, meta: &FileMeta) -> bool {
        if !self.filters.size_range.contains(meta.size) {
            return false;
        }

        if self.mtime_since.is_some_and(|since| meta.mtime.is_some_and(|mtime| mtime < since)) {
            return false;
        }

        if self.mtime_until.is_some_and(|until| meta.mtime.is_some_and(|mtime| mtime > until)) {
            return false;
        }

        true
    }
}
