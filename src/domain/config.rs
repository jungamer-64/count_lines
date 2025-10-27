mod filters;

use crate::domain::grouping::{ByMode, Granularity};
use crate::domain::options::{OutputFormat, SortKey};
use crate::foundation::util::logical_absolute;
use anyhow::Result;
use chrono::{DateTime, Local};
use std::path::PathBuf;

pub use filters::{FilterOptions, Filters};

pub type GlobPattern = glob::Pattern;

/// Keys used for grouping summary output.
#[derive(Debug, Clone, Copy)]
pub enum ByKey {
    Ext,
    Dir(usize),
    Mtime(Granularity),
}

/// Options required to assemble a [`Config`].
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct ConfigOptions {
    pub format: OutputFormat,
    pub sort_specs: Vec<(SortKey, bool)>,
    pub top_n: Option<usize>,
    pub by: Vec<ByMode>,
    pub summary_only: bool,
    pub total_only: bool,
    pub by_limit: Option<usize>,
    pub filters: FilterOptions,
    pub hidden: bool,
    pub follow: bool,
    pub use_git: bool,
    pub jobs: Option<usize>,
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
    pub compare: Option<(PathBuf, PathBuf)>,
}

/// Top-level configuration derived from structured options.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
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
    pub compare: Option<(PathBuf, PathBuf)>,
}

impl Config {
    pub fn from_options(options: ConfigOptions) -> Result<Self> {
        let filters = Filters::from_options(&options.filters)?;
        let jobs = options.jobs.unwrap_or_else(num_cpus::get).max(1);
        let paths = if options.paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            options.paths
        };
        let by_modes = options
            .by
            .into_iter()
            .filter(|b| !matches!(b, ByMode::None))
            .map(convert_by_mode)
            .collect();

        Ok(Self {
            format: options.format,
            sort_specs: options.sort_specs,
            top_n: options.top_n,
            by_modes,
            summary_only: options.summary_only,
            total_only: options.total_only,
            by_limit: options.by_limit,
            filters,
            hidden: options.hidden,
            follow: options.follow,
            use_git: options.use_git,
            jobs,
            no_default_prune: options.no_default_prune,
            abs_path: options.abs_path,
            abs_canonical: options.abs_canonical,
            trim_root: options.trim_root.map(|p| logical_absolute(&p)),
            words: options.words,
            count_newlines_in_chars: options.count_newlines_in_chars,
            text_only: options.text_only,
            fast_text_detect: options.fast_text_detect,
            files_from: options.files_from,
            files_from0: options.files_from0,
            paths,
            mtime_since: options.mtime_since,
            mtime_until: options.mtime_until,
            total_row: options.total_row,
            progress: options.progress,
            ratio: options.ratio,
            output: options.output,
            strict: options.strict,
            compare: options.compare,
        })
    }
}

fn convert_by_mode(mode: ByMode) -> ByKey {
    match mode {
        ByMode::Ext => ByKey::Ext,
        ByMode::Dir(d) => ByKey::Dir(d),
        ByMode::Mtime(g) => ByKey::Mtime(g),
        ByMode::None => unreachable!(),
    }
}
