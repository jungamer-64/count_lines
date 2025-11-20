// tests/unit/application/queries_test.rs
use std::path::PathBuf;

use count_lines_core::{
    application::{ConfigQueryService, FilterOptions},
    domain::{
        config::ByKey,
        grouping::ByMode,
        options::SortKey,
    },
};

#[path = "../../common/mod.rs"]
mod common;
use common::ConfigOptionsBuilder;

#[test]
fn build_sets_defaults_and_enables_words_when_required() {
    let filters = FilterOptions { min_words: Some(5), ..Default::default() };

    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .by(vec![ByMode::Ext, ByMode::None])
        .filters(filters)
        .abs_canonical()
        .sort_specs(vec![(SortKey::Lines, true)])
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.words, "word counting should activate for min_words filter");
    assert!(config.abs_path, "abs_canonical should imply abs_path");
    assert_eq!(config.paths, vec![PathBuf::from(".")]);
    assert!(matches!(config.by_modes.as_slice(), [ByKey::Ext]));
    assert!(config.jobs >= 1);
}

#[test]
fn build_errors_on_invalid_filter_expression() {
    let filters = FilterOptions { include: vec!["[".into()], ..Default::default() };

    let options = ConfigOptionsBuilder::new()
        .paths(vec![PathBuf::from("src")])
        .jobs(0)
        .no_default_prune(false)
        .filters(filters)
        .build();

    let err = ConfigQueryService::build(options).expect_err("invalid filter should error");
    assert!(err.to_string().contains("Invalid pattern"));
}

#[test]
fn size_sort_does_not_enable_word_counting() {
    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .sort_specs(vec![(SortKey::Size, true)])
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(!config.words, "size sorting should not enable word counting");
    assert_eq!(config.sort_specs, vec![(SortKey::Size, true)]);
}

#[test]
fn watch_enables_incremental_and_defaults_interval() {
    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .watch(true)
        .watch_interval(3)
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.watch, "watch should be enabled");
    assert!(config.incremental, "watch mode should force incremental execution");
    assert_eq!(config.watch_interval, std::time::Duration::from_secs(3));
}

#[test]
fn watch_interval_clamps_to_minimum() {
    let options =
        ConfigOptionsBuilder::new().paths(vec![]).jobs(0).no_default_prune(false).watch_interval(0).build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.watch_interval, std::time::Duration::from_secs(1));
}

#[test]
fn jobs_option_is_clamped_to_at_least_one() {
    let options = ConfigOptionsBuilder::new().paths(vec![]).jobs(0).no_default_prune(false).build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.jobs, 1);
}

#[test]
fn jobs_and_enumerator_threads_are_clamped_to_upper_bound() {
    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(10_000)
        .enumerator_threads(2_000)
        .no_default_prune(false)
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.jobs <= 512);
    assert_eq!(config.enumerator_threads, Some(512));
}

#[test]
fn watch_interval_is_capped_to_24h() {
    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .watch(true)
        .watch_interval(1_000_000)
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.watch_interval, std::time::Duration::from_secs(86_400));
}

#[test]
fn force_exts_are_lowercased_and_deduped() {
    let filters = FilterOptions {
        force_text_exts: vec!["MD".into(), "md".into()],
        force_binary_exts: vec!["DaT".into(), "dat".into()],
        ..Default::default()
    };

    let options =
        ConfigOptionsBuilder::new().paths(vec![]).jobs(0).no_default_prune(false).filters(filters).build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.filters.force_text_exts, vec!["md"]);
    assert_eq!(config.filters.force_binary_exts, vec!["dat"]);
}

#[test]
fn enumerator_controls_are_populated() {
    let filters = FilterOptions {
        overrides_include: vec!["dist/**".into()],
        overrides_exclude: vec!["build/**".into()],
        force_text_exts: vec!["Md".into()],
        force_binary_exts: vec!["DATA".into()],
        exclude_dir_only: vec!["**/generated/**".into()],
        ..Default::default()
    };

    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .case_insensitive_dedup()
        .respect_gitignore(false)
        .max_depth(5)
        .enumerator_threads(7)
        .use_ignore_overrides()
        .filters(filters)
        .build();

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
    let filters = FilterOptions { ext: Some(".rs,  .JS ,, ".into()), ..Default::default() };

    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .cache_dir(PathBuf::from("tmp/cache"))
        .filters(filters)
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    let ext_filters: std::collections::HashSet<_> = config.filters.ext_filters.iter().cloned().collect();
    let expected: std::collections::HashSet<_> = ["rs", "js"].into_iter().map(String::from).collect();

    assert!(config.cache_dir.as_ref().map(|p| p.is_absolute()).unwrap_or(false));
    assert_eq!(ext_filters, expected);
}

#[test]
fn paths_are_preserved_when_provided() {
    let options = ConfigOptionsBuilder::new()
        .paths(vec![PathBuf::from("src"), PathBuf::from("tests")])
        .jobs(0)
        .no_default_prune(false)
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert_eq!(config.paths, vec![PathBuf::from("src"), PathBuf::from("tests")]);
}

#[test]
fn sorting_by_words_enables_word_counting() {
    let options = ConfigOptionsBuilder::new()
        .paths(vec![])
        .jobs(0)
        .no_default_prune(false)
        .sort_specs(vec![(SortKey::Words, false)])
        .build();

    let config = ConfigQueryService::build(options).expect("config builds");
    assert!(config.words, "sorting by words should enable word counting");
}
