use crate::args::Args;
use crate::options::{ByMode, OutputFormat, OutputMode, SortKey, WatchOutput};
use std::path::PathBuf;
use std::time::Duration;

/// Resource limits for file processing.
///
/// Provides protection against denial-of-service attacks and handles
/// edge cases with extremely large files or deeply nested structures.
///
/// # Example
///
/// ```rust,ignore
/// use count_lines_cli::config::ResourceLimits;
///
/// let limits = ResourceLimits::default();
/// assert_eq!(limits.max_file_size, 100 * 1024 * 1024); // 100MB
/// ```
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    /// Maximum file size in bytes.
    ///
    /// Files larger than this will be skipped with a warning.
    /// Default: 100MB (104,857,600 bytes)
    pub max_file_size: u64,

    /// Maximum line length in characters.
    ///
    /// Lines longer than this may be truncated or cause the file to be skipped.
    /// Default: 1,000,000 characters
    pub max_line_length: usize,

    /// Maximum nesting depth for block comments.
    ///
    /// Prevents stack overflow from deeply nested comment structures.
    /// Default: 1,000 levels
    pub max_nested_depth: usize,

    /// Timeout for processing a single file.
    ///
    /// Files taking longer than this will be aborted.
    /// Default: 30 seconds
    pub timeout: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_line_length: 1_000_000,       // 1M characters
            max_nested_depth: 1000,           // 1000 levels
            timeout: Duration::from_secs(30), // 30 seconds
        }
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct WalkOptions {
    pub roots: Vec<PathBuf>,
    pub threads: usize,
    pub hidden: bool,
    pub git_ignore: bool,
    pub max_depth: Option<usize>,
    pub follow_links: bool,
    pub override_include: Vec<String>,
    pub override_exclude: Vec<String>,
    pub case_insensitive_dedup: bool,
    pub files_from: Option<PathBuf>,
    pub files_from0: Option<PathBuf>,
    pub types: Option<ignore::types::Types>,
}

#[derive(Debug, Clone, Default)]
pub struct FilterConfig {
    pub allow_ext: Vec<String>,
    pub deny_ext: Vec<String>, // Maybe logic handles this

    pub min_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub min_chars: Option<usize>,
    pub max_chars: Option<usize>,
    pub min_words: Option<usize>,
    pub max_words: Option<usize>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,

    pub mtime_since: Option<chrono::DateTime<chrono::Local>>,
    pub mtime_until: Option<chrono::DateTime<chrono::Local>>,

    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub map_ext: hashbrown::HashMap<String, String>,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    pub walk: WalkOptions,
    pub filter: FilterConfig,

    pub format: OutputFormat,
    pub sort: Vec<(SortKey, bool)>,
    pub top_n: Option<usize>,
    pub by: Vec<ByMode>,
    pub output_mode: OutputMode,
    pub by_limit: Option<usize>,
    pub total_row: bool,
    pub count_newlines_in_chars: bool,
    pub progress: bool,
    pub ratio: bool,
    pub output_path: Option<PathBuf>,

    pub count_words: bool,
    pub count_sloc: bool,

    pub strict: bool,
    pub incremental: bool,
    pub cache_dir: Option<PathBuf>,
    pub verify_cache: bool,
    pub clear_cache: bool,
    pub watch: bool,
    pub watch_interval: Duration,
    pub watch_output: WatchOutput,

    pub compare: Option<(PathBuf, PathBuf)>,
}

impl From<Args> for Config {
    fn from(args: Args) -> Self {
        // Resolve words/sloc dependencies
        let count_words = args.filter.words
            || args.filter.min_words.is_some()
            || args.filter.max_words.is_some()
            || args
                .output
                .sort
                .0
                .iter()
                .any(|(k, _)| matches!(k, SortKey::Words));

        let count_sloc = args.filter.sloc
            || args
                .output
                .sort
                .0
                .iter()
                .any(|(k, _)| matches!(k, SortKey::Sloc));

        let walk = WalkOptions::from_scan_and_paths(args.scan, args.paths);
        let filter = FilterConfig::from(args.filter);

        // Handle compare tuple
        let compare = args
            .comparison
            .compare
            .filter(|files| files.len() == 2)
            .map(|files| (files[0].clone(), files[1].clone()));

        Self {
            walk,
            filter,
            format: args.output.format,
            sort: args.output.sort.0,
            top_n: args.output.top,
            by: args.output.by,
            output_mode: args.output.output_mode,
            by_limit: args.output.by_limit,
            total_row: args.output.total_row,
            count_newlines_in_chars: args.output.count_newlines_in_chars,
            progress: args.output.progress,
            ratio: args.output.ratio,
            output_path: args.output.output,
            count_words,
            count_sloc,
            strict: args.behavior.strict,
            incremental: args.behavior.incremental,
            cache_dir: args.behavior.cache_dir,
            verify_cache: args.behavior.cache_verify,
            clear_cache: args.behavior.clear_cache,
            watch: args.behavior.watch,
            watch_interval: std::time::Duration::from_secs(
                args.behavior.watch_interval.unwrap_or(1),
            ),
            watch_output: args.behavior.watch_output,
            compare,
        }
    }
}

impl WalkOptions {
    fn from_scan_and_paths(scan: crate::args::ScanOptions, paths: Vec<PathBuf>) -> Self {
        let walk_threads = scan
            .walk_threads
            .or(scan.jobs)
            .unwrap_or_else(num_cpus::get);

        let roots = if paths.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            paths
        };

        Self {
            roots,
            threads: walk_threads,
            hidden: scan.hidden,
            git_ignore: !scan.no_gitignore, // Respect .gitignore by default
            max_depth: scan.max_depth,
            follow_links: scan.follow,
            override_include: scan.override_include,
            override_exclude: scan.override_exclude,
            case_insensitive_dedup: scan.case_insensitive_dedup,
            files_from: scan.files_from,
            files_from0: scan.files_from0,
            types: None,
        }
    }
}

impl From<crate::args::FilterOptions> for FilterConfig {
    fn from(opts: crate::args::FilterOptions) -> Self {
        Self {
            allow_ext: opts.ext,
            deny_ext: vec![],
            min_lines: opts.min_lines,
            max_lines: opts.max_lines,
            min_chars: opts.min_chars,
            max_chars: opts.max_chars,
            min_words: opts.min_words,
            max_words: opts.max_words,
            min_size: opts.min_size.map(|s| s.0),
            max_size: opts.max_size.map(|s| s.0),
            mtime_since: opts.mtime_since.map(|d| d.0),
            mtime_until: opts.mtime_until.map(|d| d.0),
            include_patterns: opts.include,
            exclude_patterns: opts.exclude,
            map_ext: opts.map_ext.into_iter().collect(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            walk: WalkOptions::default(),
            filter: FilterConfig::default(),
            format: OutputFormat::Table, // Table seems to be default or common
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
