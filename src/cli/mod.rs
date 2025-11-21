// src/cli/mod.rs
mod args;
mod args_groups;
mod parsers;
mod value_enum;

pub use args::Args;
use clap::Parser;
use count_lines_core::{
    application::{ConfigOptions, ConfigQueryService, FilterOptions},
    domain::config::Config,
    error::Result,
};

/// Build `FilterOptions` from parsed CLI args without taking ownership of `args`.
fn make_filter_options(args: &Args) -> FilterOptions {
    // CLI では `--ext rs,js --ext ts` のような複数指定を許容する。
    // clap の value_delimiter ですでに分割されているので、そのまま複製する。
    let ext_values = args.filter.ext.clone();
    FilterOptions {
        include: args.filter.include.clone(),
        exclude: args.filter.exclude.clone(),
        include_path: args.filter.include_path.clone(),
        exclude_path: args.filter.exclude_path.clone(),
        exclude_dir: args.filter.exclude_dir.clone(),
        exclude_dir_only: args.filter.exclude_dir_only.clone(),
        overrides_include: args.scan.override_include.clone(),
        overrides_exclude: args.scan.override_exclude.clone(),
        force_text_exts: args.filter.force_text_ext.clone(),
        force_binary_exts: args.filter.force_binary_ext.clone(),
        ext: ext_values,
        min_size: args.filter.min_size.map(|s| s.0),
        max_size: args.filter.max_size.map(|s| s.0),
        min_lines: args.filter.min_lines,
        max_lines: args.filter.max_lines,
        min_chars: args.filter.min_chars,
        max_chars: args.filter.max_chars,
        min_words: args.filter.min_words,
        max_words: args.filter.max_words,
        filter: args.filter.filter.clone(),
    }
}

fn make_compare_tuple(args: &Args) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    args.comparison.compare.as_ref().and_then(|v| if v.len() == 2 { Some((v[0].clone(), v[1].clone())) } else { None })
}

/// Build `ConfigOptions` from CLI args and precomputed pieces.
fn make_config_options(
    args: &Args,
    compare_tuple: Option<(std::path::PathBuf, std::path::PathBuf)>,
    filters: FilterOptions,
) -> ConfigOptions {
    ConfigOptions {
        format: args.output.format.into(),
        sort_specs: args.output.sort.0.clone(),
        top_n: args.output.top,
        by: args.output.by.clone(),
        summary_only: args.output.summary_only,
        total_only: args.output.total_only,
        by_limit: args.output.by_limit,
        filters,
        hidden: args.scan.hidden,
        follow: args.scan.follow,
        use_git: args.scan.git,
        respect_gitignore: !args.scan.no_gitignore,
        case_insensitive_dedup: args.scan.case_insensitive_dedup,
        max_depth: args.scan.max_depth,
        enumerator_threads: args.scan.walk_threads,
        jobs: args.scan.jobs,
        no_default_prune: args.scan.no_default_prune,
        abs_path: args.path.abs_path,
        abs_canonical: args.path.abs_canonical,
        trim_root: args.path.trim_root.clone(),
        words: args.filter.words,
        count_newlines_in_chars: args.output.count_newlines_in_chars,
        text_only: args.scan.text_only,
        fast_text_detect: args.scan.fast_text_detect,
        files_from: args.scan.files_from.clone(),
        files_from0: args.scan.files_from0.clone(),
        paths: args.paths.clone(),
        mtime_since: args.filter.mtime_since.map(|d| d.0),
        mtime_until: args.filter.mtime_until.map(|d| d.0),
        total_row: args.output.total_row,
        progress: args.output.progress,
        ratio: args.output.ratio,
        output: args.output.output.clone(),
        strict: args.behavior.strict,
        incremental: args.behavior.incremental,
        cache_dir: args.behavior.cache_dir.clone(),
        cache_verify: args.behavior.cache_verify,
        clear_cache: args.behavior.clear_cache,
        watch: args.behavior.watch,
        watch_interval: args.behavior.watch_interval,
        watch_output: args.behavior.watch_output.into(),
        compare: compare_tuple,
    }
}

/// Parse CLI arguments and materialise a domain [`Config`].
///
/// # Errors
///
/// Returns `Err` when translating the parsed arguments into a domain
/// configuration fails. Invalid CLI input is reported by `clap` before
/// reaching this function.
pub fn load_config() -> Result<Config> {
    let args = Args::parse();
    build_config(&args)
}

/// Convert parsed CLI arguments into a domain configuration.
///
/// # Errors
///
/// Returns `Err` when `ConfigQueryService` cannot build a `Config` from the
/// provided options.
pub fn build_config(args: &Args) -> Result<Config> {
    let filter_options = make_filter_options(args);
    let compare_tuple = make_compare_tuple(args);
    let options = make_config_options(args, compare_tuple, filter_options);
    ConfigQueryService::build(options)
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use clap::{Parser, error::ErrorKind};
    use count_lines_core::domain::options::SortKey;

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
        assert_eq!(config.sort_specs, vec![(SortKey::Size, true)]);
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
    fn ext_flag_accepts_multiple_forms() {
        let args = Args::parse_from(["count_lines", "--ext", "rs,JS", "--ext", ".ts"]);
        let config = build_config(&args).expect("config builds");
        let exts: std::collections::HashSet<_> = config.filters.ext_filters.iter().cloned().collect();
        let expected = ["rs", "js", "ts"]
            .into_iter()
            .map(std::string::String::from)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(exts, expected);
    }

    #[test]
    fn cli_maps_enumerator_controls() {
        let args = Args::parse_from([
            "count_lines",
            "--no-gitignore",
            "--case-insensitive-dedup",
            "--max-depth",
            "3",
            "--walk-threads",
            "4",
            "--override-include",
            "dist/**",
            "--override-exclude",
            "build/**",
            "--force-text-ext",
            "LOG",
            "--force-binary-ext",
            "Dat",
            "--exclude-dir-only",
            "generated/**",
        ]);
        let config = build_config(&args).expect("config builds");

        assert!(!config.respect_gitignore, "--no-gitignore should disable respect_gitignore");
        assert!(config.case_insensitive_dedup, "case insensitive dedup flag should propagate");
        assert_eq!(config.max_depth, Some(3));
        assert_eq!(config.enumerator_threads, Some(4));
        assert!(config.use_ignore_overrides, "override patterns should enable overrides");
        assert_eq!(config.filters.overrides_include, vec!["dist/**".to_string()]);
        assert_eq!(config.filters.overrides_exclude, vec!["build/**".to_string()]);
        assert_eq!(config.filters.force_text_exts, vec!["log".to_string()]);
        assert_eq!(config.filters.force_binary_exts, vec!["dat".to_string()]);
        let dir_only: Vec<_> =
            config.filters.exclude_dirs_only.iter().map(|g| g.pattern().to_string()).collect();
        assert_eq!(dir_only, vec!["generated/**".to_string()]);
    }

    #[test]
    fn watch_flag_enables_incremental_and_defaults_interval() {
        let args = Args::parse_from(["count_lines", "--watch"]);
        let config = build_config(&args).expect("config builds");
        assert!(config.watch);
        assert!(config.incremental, "watch should force incremental mode");
        assert_eq!(config.watch_interval, Duration::from_secs(1));
    }

    #[test]
    fn clap_rejects_zero_top() {
        let err = Args::try_parse_from(["count_lines", "--top", "0"]).expect_err("clap should reject zero");
        assert_eq!(err.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn clap_rejects_out_of_range_jobs() {
        let zero = Args::try_parse_from(["count_lines", "--jobs", "0"]).expect_err("zero invalid");
        assert_eq!(zero.kind(), ErrorKind::ValueValidation);

        let high = Args::try_parse_from(["count_lines", "--jobs", "999"]).expect_err("too many jobs invalid");
        assert_eq!(high.kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn make_compare_tuple_requires_exactly_two_paths() {
        let mut args = Args::parse_from(["count_lines"]);
        args.comparison.compare = Some(vec![PathBuf::from("only.json")]);

        assert!(make_compare_tuple(&args).is_none(), "single compare path should be ignored");
    }

    #[test]
    fn make_compare_tuple_returns_pair_when_valid() {
        let args = Args::parse_from(["count_lines", "--compare", "old.json", "new.json"]);
        let tuple = make_compare_tuple(&args).expect("should produce tuple");
        assert_eq!(tuple.0, PathBuf::from("old.json"));
        assert_eq!(tuple.1, PathBuf::from("new.json"));
    }
}
