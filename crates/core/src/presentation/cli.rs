mod args;
mod parsers;
mod value_enum;

use clap::Parser;

use crate::{
    application::{ConfigOptions, ConfigQueryService, FilterOptions},
    domain::config::Config,
    error::{PresentationError, Result},
};

fn validate_numeric_args(
    top: Option<usize>,
    by_limit: Option<usize>,
    jobs: Option<usize>,
    watch_interval: Option<u64>,
) -> Result<()> {
    validate_at_least_one("--top", top)?;
    validate_at_least_one("--by-limit", by_limit)?;
    validate_jobs("--jobs", jobs)?;
    validate_watch_interval("--watch-interval", watch_interval)?;
    Ok(())
}

fn validate_at_least_one(flag: &str, value: Option<usize>) -> Result<()> {
    if value == Some(0) {
        return Err(PresentationError::InvalidValue {
            flag: flag.to_string(),
            value: "0".to_string(),
            reason: "must be at least 1".to_string(),
        }
        .into());
    }
    Ok(())
}

fn validate_jobs(flag: &str, jobs: Option<usize>) -> Result<()> {
    match jobs {
        Some(0) => Err(PresentationError::InvalidValue {
            flag: flag.to_string(),
            value: "0".to_string(),
            reason: "must be between 1 and 512".to_string(),
        }
        .into()),
        Some(j) if j > 512 => Err(PresentationError::InvalidValue {
            flag: flag.to_string(),
            value: j.to_string(),
            reason: "must be between 1 and 512".to_string(),
        }
        .into()),
        _ => Ok(()),
    }
}

fn validate_watch_interval(flag: &str, interval: Option<u64>) -> Result<()> {
    if interval == Some(0) {
        return Err(PresentationError::InvalidValue {
            flag: flag.to_string(),
            value: "0".to_string(),
            reason: "must be at least 1".to_string(),
        }
        .into());
    }
    Ok(())
}

/// Build `FilterOptions` from parsed CLI args without taking ownership of `args`.
fn make_filter_options(args: &Args) -> FilterOptions {
    FilterOptions {
        include: args.include.clone(),
        exclude: args.exclude.clone(),
        include_path: args.include_path.clone(),
        exclude_path: args.exclude_path.clone(),
        exclude_dir: args.exclude_dir.clone(),
        ext: args.ext.clone(),
        min_size: args.min_size.map(|s| s.0),
        max_size: args.max_size.map(|s| s.0),
        min_lines: args.min_lines,
        max_lines: args.max_lines,
        min_chars: args.min_chars,
        max_chars: args.max_chars,
        min_words: args.min_words,
        max_words: args.max_words,
        filter: args.filter.clone(),
    }
}

fn make_compare_tuple(args: &Args) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    args.compare
        .as_ref()
        .and_then(|v| if v.len() == 2 { Some((v[0].clone(), v[1].clone())) } else { None })
}

/// Build `ConfigOptions` from CLI args and precomputed pieces.
fn make_config_options(
    args: &Args,
    compare_tuple: Option<(std::path::PathBuf, std::path::PathBuf)>,
    filters: FilterOptions,
) -> ConfigOptions {
    ConfigOptions {
        format: args.format,
    sort_specs: args.sort.0.clone(),
    top_n: args.top,
    by: args.by.clone(),
        summary_only: args.summary_only,
        total_only: args.total_only,
        by_limit: args.by_limit,
        filters,
        hidden: args.hidden,
        follow: args.follow,
        use_git: args.git,
        jobs: args.jobs,
        no_default_prune: args.no_default_prune,
        abs_path: args.abs_path,
        abs_canonical: args.abs_canonical,
    trim_root: args.trim_root.clone(),
        words: args.words,
        count_newlines_in_chars: args.count_newlines_in_chars,
        text_only: args.text_only,
        fast_text_detect: args.fast_text_detect,
    files_from: args.files_from.clone(),
    files_from0: args.files_from0.clone(),
    paths: args.paths.clone(),
        mtime_since: args.mtime_since.map(|d| d.0),
        mtime_until: args.mtime_until.map(|d| d.0),
        total_row: args.total_row,
        progress: args.progress,
        ratio: args.ratio,
    output: args.output.clone(),
        strict: args.strict,
        incremental: args.incremental,
        cache_dir: args.cache_dir.clone(),
        cache_verify: args.cache_verify,
        clear_cache: args.clear_cache,
        watch: args.watch,
        watch_interval: args.watch_interval,
        watch_output: args.watch_output,
        compare: compare_tuple,
    }
}

pub use args::Args;

/// Parse CLI arguments and materialise a domain [`Config`].
///
/// # Errors
///
/// Returns `Err` when the parsed arguments are invalid (for example numeric
/// flags outside their allowed ranges) or when building the domain
/// `Config` from the parsed options fails.
pub fn load_config() -> Result<Config> {
    let args = Args::parse();
    build_config(&args)
}

/// Convert parsed CLI arguments into a domain configuration.
///
/// # Errors
///
/// Returns `Err` when argument validation fails or when `ConfigQueryService`
/// cannot build a `Config` from the provided options.
pub fn build_config(args: &Args) -> Result<Config> {
    // Validate numeric arguments
    validate_numeric_args(args.top, args.by_limit, args.jobs, args.watch_interval)?;

    // Use helpers which avoid taking ownership of `args` here to keep this function short.
    let filter_options = make_filter_options(args);
    let compare_tuple = make_compare_tuple(args);
    let options = make_config_options(args, compare_tuple, filter_options);
    ConfigQueryService::build(options)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn min_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--min-words", "5"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.words, "min-words should trigger word counting");
    }

    #[test]
    fn sort_by_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--sort", "words:desc"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.words, "sorting by words should trigger word counting");
    }

    #[test]
    fn filter_expression_mentioning_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--filter", "words > 10"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.words, "filter expressions referencing words should trigger word counting");
    }

    #[test]
    fn sort_by_size_is_accepted_without_enabling_words() {
        let args = Args::parse_from(["count_lines", "--sort", "size:desc"]);
        let config = build_config(&args).expect("config builds");
        assert!(!config.words, "sorting by size should not enable word counting");
        assert_eq!(config.sort_specs, vec![(crate::domain::options::SortKey::Size, true)]);
    }

    #[test]
    fn abs_canonical_implies_abs_path() {
        let args = Args::parse_from(["count_lines", "--abs-canonical"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.abs_canonical);
        assert!(config.abs_path, "--abs-canonical should imply absolute path formatting");
    }

    #[test]
    fn incremental_flag_enables_cache_usage() {
        let args = Args::parse_from(["count_lines", "--incremental"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.incremental, "--incremental should enable incremental mode");
    }

    #[test]
    fn cache_dir_is_normalised_to_absolute_path() {
        let args = Args::parse_from(["count_lines", "--cache-dir", "./tmp/cache"]);
        let config = build_config(&args).expect("config builds");
        let cache_dir = config.cache_dir.expect("cache dir should be set");
        assert!(cache_dir.is_absolute(), "cache dir should be normalised to an absolute path");
    }

    #[test]
    fn watch_flag_enables_incremental_and_defaults_interval() {
        let args = Args::parse_from(["count_lines", "--watch"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.watch);
        assert!(config.incremental, "watch should force incremental mode");
        assert_eq!(config.watch_interval, std::time::Duration::from_secs(1));
    }
}
