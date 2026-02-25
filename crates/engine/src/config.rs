use crate::options::{ByMode, OutputFormat, OutputMode, SortKey, WatchOutput};
use derive_builder::Builder;
use std::path::PathBuf;
use std::time::Duration;


#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct WalkOptions {
    #[builder(default)]
    pub roots: Vec<PathBuf>,
    #[builder(default = "1")]
    pub threads: usize,
    #[builder(default)]
    pub hidden: bool,
    #[builder(default = "true")]
    pub git_ignore: bool,
    #[builder(default)]
    pub max_depth: Option<usize>,
    #[builder(default)]
    pub follow_links: bool,
    #[builder(default)]
    pub override_include: Vec<String>,
    #[builder(default)]
    pub override_exclude: Vec<String>,
    #[builder(default)]
    pub case_insensitive_dedup: bool,
    #[builder(default)]
    pub files_from: Option<PathBuf>,
    #[builder(default)]
    pub files_from0: Option<PathBuf>,
    #[builder(default, setter(strip_option))]
    pub types: Option<ignore::types::Types>,
}

impl Default for WalkOptions {
    fn default() -> Self {
        Self {
            roots: vec![],
            threads: 1,
            hidden: false,
            git_ignore: true,
            max_depth: None,
            follow_links: false,
            override_include: vec![],
            override_exclude: vec![],
            case_insensitive_dedup: false,
            files_from: None,
            files_from0: None,
            types: None,
        }
    }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(setter(into))]
pub struct FilterConfig {
    #[builder(default)]
    pub allow_ext: Vec<String>,
    #[builder(default)]
    pub deny_ext: Vec<String>,

    #[builder(default)]
    pub min_lines: Option<usize>,
    #[builder(default)]
    pub max_lines: Option<usize>,
    #[builder(default)]
    pub min_chars: Option<usize>,
    #[builder(default)]
    pub max_chars: Option<usize>,
    #[builder(default)]
    pub min_words: Option<usize>,
    #[builder(default)]
    pub max_words: Option<usize>,
    #[builder(default)]
    pub min_size: Option<u64>,
    #[builder(default)]
    pub max_size: Option<u64>,

    #[builder(default)]
    pub mtime_since: Option<chrono::DateTime<chrono::Local>>,
    #[builder(default)]
    pub mtime_until: Option<chrono::DateTime<chrono::Local>>,

    #[builder(default)]
    pub include_patterns: Vec<String>,
    #[builder(default)]
    pub exclude_patterns: Vec<String>,
    #[builder(default)]
    pub map_ext: hashbrown::HashMap<String, String>,
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Config {
    #[builder(default)]
    pub walk: WalkOptions,
    #[builder(default)]
    pub filter: FilterConfig,

    #[builder(default = "OutputFormat::Table")]
    pub format: OutputFormat,
    #[builder(default)]
    pub sort: Vec<(SortKey, bool)>,
    #[builder(default)]
    pub top_n: Option<usize>,
    #[builder(default)]
    pub by: Vec<ByMode>,
    #[builder(default)]
    pub output_mode: OutputMode,
    #[builder(default)]
    pub by_limit: Option<usize>,
    #[builder(default)]
    pub total_row: bool,
    #[builder(default)]
    pub count_newlines_in_chars: bool,
    #[builder(default)]
    pub progress: bool,
    #[builder(default)]
    pub ratio: bool,
    #[builder(default)]
    pub output_path: Option<PathBuf>,

    #[builder(default)]
    pub count_words: bool,
    #[builder(default)]
    pub count_sloc: bool,

    #[builder(default)]
    pub strict: bool,
    #[builder(default)]
    pub incremental: bool,
    #[builder(default)]
    pub cache_dir: Option<PathBuf>,
    #[builder(default)]
    pub verify_cache: bool,
    #[builder(default)]
    pub clear_cache: bool,
    #[builder(default)]
    pub watch: bool,
    #[builder(default = "Duration::from_secs(1)")]
    pub watch_interval: Duration,
    #[builder(default = "WatchOutput::Full")]
    pub watch_output: WatchOutput,

    #[builder(default)]
    pub compare: Option<(PathBuf, PathBuf)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            walk: WalkOptions::default(),
            filter: FilterConfig::default(),
            format: OutputFormat::Table,
            sort: vec![],
            top_n: None,
            by: vec![],
            output_mode: OutputMode::default(),
            by_limit: None,
            total_row: false,
            count_newlines_in_chars: false,
            progress: false,
            ratio: false,
            output_path: None,
            count_words: false,
            count_sloc: false,
            strict: false,
            incremental: false,
            cache_dir: None,
            verify_cache: false,
            clear_cache: false,
            watch: false,
            watch_interval: Duration::from_secs(1),
            watch_output: WatchOutput::Full,
            compare: None,
        }
    }
}
