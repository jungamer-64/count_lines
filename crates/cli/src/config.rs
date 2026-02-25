// crates/cli/src/config.rs
use crate::args::Args;
use crate::options::{self, SortKey};
pub use count_lines_engine::config::{
    Config, ConfigBuilder, FilterConfig, FilterConfigBuilder, WalkOptions, WalkOptionsBuilder,
};
use count_lines_engine::options as engine_options;
use std::path::PathBuf;
use std::time::Duration;

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

        let walk = walk_options_from_args(&args);
        let filter = filter_config_from_args(&args);

        // Handle compare tuple
        let compare = args
            .comparison
            .compare
            .as_ref()
            .filter(|files| files.len() == 2)
            .map(|files| (files[0].clone(), files[1].clone()));

        // Convert enums via From impls
        let format: engine_options::OutputFormat = args.output.format.into();
        let output_mode: engine_options::OutputMode = args.output.output_mode.into();
        let watch_output: engine_options::WatchOutput = args.behavior.watch_output.into();
        let by_mode: Vec<_> = args
            .output
            .by
            .into_iter()
            .map(engine_options::ByMode::from)
            .collect();
        let sort: Vec<_> = args
            .output
            .sort
            .0
            .into_iter()
            .map(|(k, d)| (engine_options::SortKey::from(k), d))
            .collect();

        ConfigBuilder::default()
            .walk(walk)
            .filter(filter)
            .format(format)
            .sort(sort)
            .top_n(args.output.top)
            .by(by_mode)
            .output_mode(output_mode)
            .by_limit(args.output.by_limit)
            .total_row(args.output.total_row)
            .count_newlines_in_chars(args.output.count_newlines_in_chars)
            .progress(args.output.progress)
            .ratio(args.output.ratio)
            .output_path(args.output.output)
            .count_words(count_words)
            .count_sloc(count_sloc)
            .strict(args.behavior.strict)
            .incremental(args.behavior.incremental)
            .cache_dir(args.behavior.cache_dir)
            .verify_cache(args.behavior.cache_verify)
            .clear_cache(args.behavior.clear_cache)
            .watch(args.behavior.watch)
            .watch_interval(Duration::from_secs(
                args.behavior.watch_interval.unwrap_or(1),
            ))
            .watch_output(watch_output)
            .compare(compare)
            .build()
            .expect("Failed to build config")
    }
}

fn walk_options_from_args(args: &Args) -> WalkOptions {
    let scan = &args.scan;
    let paths = &args.paths;

    let walk_threads = scan
        .walk_threads
        .or(scan.jobs)
        .unwrap_or_else(num_cpus::get);

    let roots = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.clone()
    };

    WalkOptionsBuilder::default()
        .roots(roots)
        .threads(walk_threads)
        .hidden(scan.hidden)
        .git_ignore(!scan.no_gitignore)
        .max_depth(scan.max_depth)
        .follow_links(scan.follow)
        .override_include(scan.override_include.clone())
        .override_exclude(scan.override_exclude.clone())
        .case_insensitive_dedup(scan.case_insensitive_dedup)
        .files_from(scan.files_from.clone())
        .files_from0(scan.files_from0.clone())
        .build()
        .expect("Failed to build walk options")
}

fn filter_config_from_args(args: &Args) -> FilterConfig {
    let opts = &args.filter;
    let map_ext: hashbrown::HashMap<String, String> = opts.map_ext.clone().into_iter().collect();

    FilterConfigBuilder::default()
        .allow_ext(opts.ext.clone())
        .min_lines(opts.min_lines)
        .max_lines(opts.max_lines)
        .min_chars(opts.min_chars)
        .max_chars(opts.max_chars)
        .min_words(opts.min_words)
        .max_words(opts.max_words)
        .min_size(opts.min_size.map(|s| s.0))
        .max_size(opts.max_size.map(|s| s.0))
        .mtime_since(opts.mtime_since.map(|d| d.0))
        .mtime_until(opts.mtime_until.map(|d| d.0))
        .include_patterns(opts.include.clone())
        .exclude_patterns(opts.exclude.clone())
        .map_ext(map_ext)
        .build()
        .expect("Failed to build filter config")
}

// From trait implementations for CLI -> Engine enum conversion

macro_rules! map_enum {
    ($from:ty, $to:ty, $($variant:ident),+ $(,)?) => {
        impl From<$from> for $to {
            fn from(f: $from) -> Self {
                match f {
                    $( <$from>::$variant => <$to>::$variant, )+
                }
            }
        }
    };
}

map_enum!(
    options::OutputFormat,
    engine_options::OutputFormat,
    Table,
    Csv,
    Tsv,
    Json,
    Yaml,
    Md,
    Jsonl
);
map_enum!(
    options::OutputMode,
    engine_options::OutputMode,
    Full,
    Summary,
    TotalOnly
);
map_enum!(
    options::WatchOutput,
    engine_options::WatchOutput,
    Full,
    Jsonl
);
map_enum!(
    options::SortKey,
    engine_options::SortKey,
    Lines,
    Chars,
    Words,
    Size,
    Name,
    Ext,
    Sloc
);
map_enum!(
    options::Granularity,
    engine_options::Granularity,
    Day,
    Week,
    Month
);

impl From<options::ByMode> for engine_options::ByMode {
    fn from(b: options::ByMode) -> Self {
        match b {
            options::ByMode::Ext => Self::Ext,
            options::ByMode::Dir(d) => Self::Dir(d),
            options::ByMode::Mtime(g) => Self::Mtime(g.into()),
        }
    }
}
