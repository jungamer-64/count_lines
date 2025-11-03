// tests/unit/application/queries_test.rs
use std::path::PathBuf;

use count_lines_core::{
    application::{ConfigOptions, ConfigQueryService, FilterOptions},
    domain::{
        config::ByKey,
        grouping::ByMode,
        options::{OutputFormat, SortKey},
    },
};

fn base_options() -> ConfigOptions {
    ConfigOptions {
        format: OutputFormat::Json,
        sort_specs: Vec::new(),
        top_n: None,
        by: vec![],
        summary_only: false,
        total_only: false,
        by_limit: None,
        filters: FilterOptions::default(),
        hidden: false,
        follow: false,
        use_git: false,
        respect_gitignore: true,
        use_ignore_overrides: false,
        case_insensitive_dedup: false,
        max_depth: None,
        enumerator_threads: None,
        jobs: None,
        no_default_prune: false,
        abs_path: false,
        abs_canonical: false,
        trim_root: None,
        words: false,
        count_newlines_in_chars: false,
        text_only: false,
        fast_text_detect: false,
        files_from: None,
        files_from0: None,
        paths: Vec::new(),
        mtime_since: None,
        mtime_until: None,
        total_row: false,
        progress: false,
        ratio: false,
        output: None,
        strict: false,
        incremental: false,
        cache_dir: None,
        cache_verify: false,
        clear_cache: false,
        watch: false,
        watch_interval: None,
        watch_output: count_lines_core::domain::options::WatchOutput::Full,
        compare: None,
    }
}

#[test]
fn build_sets_defaults_and_enables_words_when_required() {
    let mut options = base_options();
    options.by = vec![ByMode::Ext, ByMode::None];
    options.filters.min_words = Some(5);
    options.abs_canonical = true;
    options.sort_specs = vec![(SortKey::Lines, true)];

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.words, "word counting should activate for min_words filter");
    assert!(config.abs_path, "abs_canonical should imply abs_path");
    assert_eq!(config.paths, vec![PathBuf::from(".")]);
    assert!(matches!(config.by_modes.as_slice(), [ByKey::Ext]));
    assert!(config.jobs >= 1);
}

#[test]
fn build_errors_on_invalid_filter_expression() {
    let mut options = base_options();
    options.paths = vec![PathBuf::from("src")];
    options.filters.include.push("[".into());

    let err = ConfigQueryService::build(options).expect_err("invalid filter should error");
    assert!(err.to_string().contains("Invalid pattern"));
}

#[test]
fn size_sort_does_not_enable_word_counting() {
    let mut options = base_options();
    options.sort_specs = vec![(SortKey::Size, true)];

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(!config.words, "size sorting should not enable word counting");
    assert_eq!(config.sort_specs, vec![(SortKey::Size, true)]);
}

#[test]
fn watch_enables_incremental_and_defaults_interval() {
    let mut options = base_options();
    options.watch = true;
    options.watch_interval = Some(3);

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.watch, "watch should be enabled");
    assert!(config.incremental, "watch mode should force incremental execution");
    assert_eq!(config.watch_interval, std::time::Duration::from_secs(3));
}

#[test]
fn watch_interval_clamps_to_minimum() {
    let mut options = base_options();
    options.watch_interval = Some(0);

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.watch_interval, std::time::Duration::from_secs(1));
}

#[test]
fn jobs_option_is_clamped_to_at_least_one() {
    let mut options = base_options();
    options.jobs = Some(0);

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.jobs, 1);
}

#[test]
fn jobs_and_enumerator_threads_are_clamped_to_upper_bound() {
    let mut options = base_options();
    options.jobs = Some(10_000);
    options.enumerator_threads = Some(2_000);
    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.jobs <= 512);
    assert_eq!(config.enumerator_threads, Some(512));
}

#[test]
fn watch_interval_is_capped_to_24h() {
    let mut options = base_options();
    options.watch = true;
    options.watch_interval = Some(1_000_000);
    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.watch_interval, std::time::Duration::from_secs(86_400));
}

#[test]
fn force_exts_are_lowercased_and_deduped() {
    let mut options = base_options();
    options.filters.force_text_exts = vec!["MD".into(), "md".into()];
    options.filters.force_binary_exts = vec!["DaT".into(), "dat".into()];
    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.filters.force_text_exts, vec!["md"]);
    assert_eq!(config.filters.force_binary_exts, vec!["dat"]);
}

#[test]
fn enumerator_controls_are_populated() {
    let mut options = base_options();
    options.case_insensitive_dedup = true;
    options.respect_gitignore = false;
    options.max_depth = Some(5);
    options.enumerator_threads = Some(7);
    options.use_ignore_overrides = true;
    options.filters.overrides_include = vec!["dist/**".into()];
    options.filters.overrides_exclude = vec!["build/**".into()];
    options.filters.force_text_exts = vec!["Md".into()];
    options.filters.force_binary_exts = vec!["DATA".into()];
    options.filters.exclude_dir_only = vec!["**/generated/**".into()];

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.case_insensitive_dedup);
    assert!(!config.respect_gitignore);
    assert!(config.use_ignore_overrides);
    assert_eq!(config.max_depth, Some(5));
    assert_eq!(config.enumerator_threads, Some(7));
    assert_eq!(config.filters.overrides_include, vec!["dist/**".to_string()]);
    assert_eq!(config.filters.overrides_exclude, vec!["build/**".to_string()]);
    assert_eq!(config.filters.force_text_exts, vec!["md".to_string()]);
    assert_eq!(config.filters.force_binary_exts, vec!["data".to_string()]);
    let dir_only: Vec<_> = config.filters.exclude_dirs_only.iter().map(|g| g.pattern().to_string()).collect();
    assert_eq!(dir_only, vec!["**/generated/**".to_string()]);
}

#[test]
fn cache_dir_is_normalised_and_extensions_are_lowercased() {
    let mut options = base_options();
    options.cache_dir = Some(PathBuf::from("tmp/cache"));
    options.filters.ext = Some(".rs,  .JS ,, ".into());

    let config = ConfigQueryService::build(options).expect("config builds");
    let ext_filters: std::collections::HashSet<_> = config.filters.ext_filters.iter().cloned().collect();
    let expected: std::collections::HashSet<_> = ["rs", "js"].into_iter().map(String::from).collect();

    assert!(config.cache_dir.as_ref().map(|p| p.is_absolute()).unwrap_or(false));
    assert_eq!(ext_filters, expected);
}

#[test]
fn paths_are_preserved_when_provided() {
    let mut options = base_options();
    options.paths = vec![PathBuf::from("src"), PathBuf::from("tests")];

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.paths, vec![PathBuf::from("src"), PathBuf::from("tests")]);
}

#[test]
fn sorting_by_words_enables_word_counting() {
    let mut options = base_options();
    options.sort_specs = vec![(SortKey::Words, false)];

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.words, "sorting by words should enable word counting");
}
