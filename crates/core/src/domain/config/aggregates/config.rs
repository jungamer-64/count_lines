use std::{path::PathBuf, time::Duration};

use chrono::{DateTime, Local};

use crate::domain::{
    config::{ByKey, Filters},
    options::{OutputFormat, SortKey, WatchOutput},
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
