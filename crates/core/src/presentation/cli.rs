mod args;
mod parsers;
mod value_enum;

use clap::Parser;

use crate::{
    application::{ConfigOptions, ConfigQueryService, FilterOptions},
    domain::config::Config,
    error::{PresentationError, Result},
};

fn validate_numeric_args(top: Option<usize>, by_limit: Option<usize>, jobs: Option<usize>) -> Result<()> {
    if let Some(t) = top
        && t == 0
    {
        return Err(PresentationError::InvalidValue {
            flag: "--top".to_string(),
            value: t.to_string(),
            reason: "must be at least 1".to_string(),
        }
        .into());
    }
    if let Some(bl) = by_limit
        && bl == 0
    {
        return Err(PresentationError::InvalidValue {
            flag: "--by-limit".to_string(),
            value: bl.to_string(),
            reason: "must be at least 1".to_string(),
        }
        .into());
    }
    if let Some(j) = jobs
        && (j == 0 || j > 512)
    {
        return Err(PresentationError::InvalidValue {
            flag: "--jobs".to_string(),
            value: j.to_string(),
            reason: "must be between 1 and 512".to_string(),
        }
        .into());
    }
    Ok(())
}

pub use args::Args;

/// Parse CLI arguments and materialise a domain [`Config`].
pub fn load_config() -> Result<Config> {
    let args = Args::parse();
    build_config(args)
}

/// Convert parsed CLI arguments into a domain configuration.
pub fn build_config(args: Args) -> Result<Config> {
    // Validate numeric arguments
    validate_numeric_args(args.top, args.by_limit, args.jobs)?;

    let filter_options = FilterOptions {
        include: args.include,
        exclude: args.exclude,
        include_path: args.include_path,
        exclude_path: args.exclude_path,
        exclude_dir: args.exclude_dir,
        ext: args.ext,
        min_size: args.min_size.map(|s| s.0),
        max_size: args.max_size.map(|s| s.0),
        min_lines: args.min_lines,
        max_lines: args.max_lines,
        min_chars: args.min_chars,
        max_chars: args.max_chars,
        min_words: args.min_words,
        max_words: args.max_words,
        filter: args.filter,
    };

    let compare_tuple =
        args.compare.and_then(|mut v| if v.len() == 2 { Some((v.remove(0), v.remove(0))) } else { None });

    let options = ConfigOptions {
        format: args.format,
        sort_specs: args.sort.0,
        top_n: args.top,
        by: args.by,
        summary_only: args.summary_only,
        total_only: args.total_only,
        by_limit: args.by_limit,
        filters: filter_options,
        hidden: args.hidden,
        follow: args.follow,
        use_git: args.git,
        jobs: args.jobs,
        no_default_prune: args.no_default_prune,
        abs_path: args.abs_path,
        abs_canonical: args.abs_canonical,
        trim_root: args.trim_root,
        words: args.words,
        count_newlines_in_chars: args.count_newlines_in_chars,
        text_only: args.text_only,
        fast_text_detect: args.fast_text_detect,
        files_from: args.files_from,
        files_from0: args.files_from0,
        paths: args.paths,
        mtime_since: args.mtime_since.map(|d| d.0),
        mtime_until: args.mtime_until.map(|d| d.0),
        total_row: args.total_row,
        progress: args.progress,
        ratio: args.ratio,
        output: args.output,
        strict: args.strict,
        incremental: args.incremental,
        cache_dir: args.cache_dir,
        compare: compare_tuple,
    };

    ConfigQueryService::build(options)
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::*;

    #[test]
    fn min_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--min-words", "5"]);
        let config = build_config(args).expect("config builds");
        assert!(config.words, "min-words should trigger word counting");
    }

    #[test]
    fn sort_by_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--sort", "words:desc"]);
        let config = build_config(args).expect("config builds");
        assert!(config.words, "sorting by words should trigger word counting");
    }

    #[test]
    fn filter_expression_mentioning_words_enables_word_counting() {
        let args = Args::parse_from(["count_lines", "--filter", "words > 10"]);
        let config = build_config(args).expect("config builds");
        assert!(config.words, "filter expressions referencing words should trigger word counting");
    }

    #[test]
    fn sort_by_size_is_accepted_without_enabling_words() {
        let args = Args::parse_from(["count_lines", "--sort", "size:desc"]);
        let config = build_config(args).expect("config builds");
        assert!(!config.words, "sorting by size should not enable word counting");
        assert_eq!(config.sort_specs, vec![(crate::domain::options::SortKey::Size, true)]);
    }

    #[test]
    fn abs_canonical_implies_abs_path() {
        let args = Args::parse_from(["count_lines", "--abs-canonical"]);
        let config = build_config(args).expect("config builds");
        assert!(config.abs_canonical);
        assert!(config.abs_path, "--abs-canonical should imply absolute path formatting");
    }

    #[test]
    fn incremental_flag_enables_cache_usage() {
        let args = Args::parse_from(["count_lines", "--incremental"]);
        let config = build_config(args).expect("config builds");
        assert!(config.incremental, "--incremental should enable incremental mode");
    }

    #[test]
    fn cache_dir_is_normalised_to_absolute_path() {
        let args = Args::parse_from(["count_lines", "--cache-dir", "./tmp/cache"]);
        let config = build_config(args).expect("config builds");
        let cache_dir = config.cache_dir.expect("cache dir should be set");
        assert!(cache_dir.is_absolute(), "cache dir should be normalised to an absolute path");
    }
}
