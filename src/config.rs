// src/config.rs
use crate::cli::{ByMode, Granularity, OutputFormat, SortKey};
use crate::util::logical_absolute;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local};
use evalexpr::Node;
use std::collections::HashSet;
use std::path::PathBuf;

pub type GlobPattern = glob::Pattern;

/// Filtering parameters derived from CLI arguments.
#[derive(Debug, Default)]
pub struct Filters {
    pub include_patterns: Vec<GlobPattern>,
    pub exclude_patterns: Vec<GlobPattern>,
    pub include_paths: Vec<GlobPattern>,
    pub exclude_paths: Vec<GlobPattern>,
    pub exclude_dirs: Vec<GlobPattern>,
    pub ext_filters: HashSet<String>,
    pub size_range: SizeRange,
    pub lines_range: Range,
    pub chars_range: Range,
    pub words_range: Range,
    pub filter_ast: Option<Node>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SizeRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Range {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl Range {
    fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }
    pub fn contains(&self, v: usize) -> bool {
        self.min.map_or(true, |m| v >= m) && self.max.map_or(true, |x| v <= x)
    }
}

impl SizeRange {
    fn new(min: Option<u64>, max: Option<u64>) -> Self {
        Self { min, max }
    }
    pub fn contains(&self, v: u64) -> bool {
        self.min.map_or(true, |m| v >= m) && self.max.map_or(true, |x| v <= x)
    }
}

impl Filters {
    /// Construct a `Filters` from CLI arguments. This parses patterns
    /// and builds an optional evalexpr AST for advanced filtering.
    pub fn from_args(args: &crate::cli::Args) -> Result<Self> {
        let filter_ast = args
            .filter
            .as_ref()
            .map(|expr| evalexpr::build_operator_tree(expr).map_err(|e| anyhow!(e)))
            .transpose()?;

        Ok(Self {
            include_patterns: crate::util::parse_patterns(&args.include)?,
            exclude_patterns: crate::util::parse_patterns(&args.exclude)?,
            include_paths: crate::util::parse_patterns(&args.include_path)?,
            exclude_paths: crate::util::parse_patterns(&args.exclude_path)?,
            exclude_dirs: crate::util::parse_patterns(&args.exclude_dir)?,
            ext_filters: Self::parse_extensions(&args.ext),
            size_range: SizeRange::new(args.min_size.map(|s| s.0), args.max_size.map(|s| s.0)),
            lines_range: Range::new(args.min_lines, args.max_lines),
            chars_range: Range::new(args.min_chars, args.max_chars),
            words_range: Range::new(args.min_words, args.max_words),
            filter_ast,
        })
    }

    fn parse_extensions(ext_arg: &Option<String>) -> HashSet<String> {
        ext_arg
            .as_ref()
            .map(|s| {
                s.split(',')
                    .map(|e| e.trim().to_lowercase())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Keys used for grouping summary output.
#[derive(Debug, Clone, Copy)]
pub enum ByKey {
    Ext,
    Dir(usize),
    Mtime(Granularity),
}

/// Top-level configuration derived from CLI arguments.
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

impl TryFrom<crate::cli::Args> for Config {
    type Error = anyhow::Error;
    fn try_from(args: crate::cli::Args) -> Result<Self> {
        let filters = Filters::from_args(&args)?;
        let jobs = args.jobs.unwrap_or_else(num_cpus::get).max(1);

        let paths = if args.paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            args.paths
        };

        let by_modes = args
            .by
            .into_iter()
            .filter(|b| !matches!(b, ByMode::None))
            .map(Self::convert_by_mode)
            .collect();

        let compare = args
            .compare
            .and_then(|v| (v.len() == 2).then(|| (v[0].clone(), v[1].clone())));

        Ok(Self {
            format: args.format,
            sort_specs: args.sort.0,
            top_n: args.top,
            by_modes,
            summary_only: args.summary_only,
            total_only: args.total_only,
            by_limit: args.by_limit,
            filters,
            hidden: args.hidden,
            follow: args.follow,
            use_git: args.git,
            jobs,
            no_default_prune: args.no_default_prune,
            abs_path: args.abs_path,
            abs_canonical: args.abs_canonical,
            trim_root: args.trim_root.map(|p| logical_absolute(&p)),
            words: args.words,
            count_newlines_in_chars: args.count_newlines_in_chars,
            text_only: args.text_only,
            fast_text_detect: args.fast_text_detect,
            files_from: args.files_from,
            files_from0: args.files_from0,
            paths,
            mtime_since: args.mtime_since.map(|d| d.0),
            mtime_until: args.mtime_until.map(|d| d.0),
            total_row: args.total_row,
            progress: args.progress,
            ratio: args.ratio,
            output: args.output,
            strict: args.strict,
            compare,
        })
    }
}

impl Config {
    fn convert_by_mode(mode: ByMode) -> ByKey {
        match mode {
            ByMode::Ext => ByKey::Ext,
            ByMode::Dir(d) => ByKey::Dir(d),
            ByMode::Mtime(g) => ByKey::Mtime(g),
            ByMode::None => unreachable!(),
        }
    }
}