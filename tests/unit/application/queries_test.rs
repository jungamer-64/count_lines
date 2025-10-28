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
