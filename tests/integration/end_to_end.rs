// tests/integration/end_to_end.rs
use std::{fs, path::Path};

use count_lines_core::{
    application::{ConfigOptions, ConfigQueryService, FilterOptions},
    domain::{
        grouping::ByMode,
        options::{OutputFormat, SortKey},
    },
    run_with_config,
};
use serde_json::Value;

#[path = "../common/mod.rs"]
mod common;
use common::{FileStatsBuilder, TempDir, TempWorkspace, assert_stats};

fn base_options(root: &Path) -> ConfigOptions {
    ConfigOptions {
        format: OutputFormat::Json,
        sort_specs: vec![],
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
        jobs: Some(1),
        no_default_prune: true,
        abs_path: false,
        abs_canonical: false,
        trim_root: None,
        words: false,
        count_newlines_in_chars: false,
        text_only: false,
        fast_text_detect: false,
        files_from: None,
        files_from0: None,
        paths: vec![root.to_path_buf()],
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

fn read_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("output exists");
    serde_json::from_str(&contents).expect("valid JSON")
}

#[test]
fn end_to_end_generates_expected_json() {
    let temp = TempDir::new("end_to_end", "count_lines_integration");
    temp.write_file("src/lib.rs", "fn main() {\n    println!(\"hello\");\n}\n");
    temp.write_file("README.md", "# Count Lines\nMore text\n");

    let output_path = temp.path().join("result.json");
    let mut options = base_options(temp.path());
    options.format = OutputFormat::Json;
    options.sort_specs = vec![(SortKey::Lines, true)];
    options.by = vec![ByMode::Ext];
    options.hidden = true;
    options.words = true;
    options.output = Some(output_path.clone());
    options.strict = true;

    let config = ConfigQueryService::build(options).expect("config builds");
    run_with_config(config).expect("run succeeds");

    let json = read_json(&output_path);

    let summary_lines = json["summary"]["lines"].as_u64().expect("summary lines present");
    assert_eq!(summary_lines, 5);

    let files = json["files"].as_array().expect("files array");
    assert_eq!(files.len(), 2);
    let exts: Vec<_> = files.iter().map(|f| f["ext"].as_str().unwrap()).collect();
    assert!(exts.contains(&"rs"));
    assert!(exts.contains(&"md"));

    let groups = json["by"].as_array().expect("groups present");
    assert_eq!(groups[0]["label"], "By Extension");
    let rows = groups[0]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
}

#[test]
fn end_to_end_with_value_objects() {
    let workspace = TempWorkspace::new("e2e_value_objects", "count_lines_integration");
    workspace.create_file("src/main.rs", "fn main() {\n    println!(\"hello world\");\n}\n");
    workspace.create_file("README.md", "# Project\n\nDescription here.\n");
    workspace.create_file("Cargo.toml", "[package]\nname = \"test\"\n");

    let output_path = workspace.path().join("output.json");

    let mut options = base_options(workspace.path());
    options.format = OutputFormat::Json;
    options.sort_specs = vec![(SortKey::Lines, true)];
    options.by = vec![ByMode::Ext];
    options.hidden = false;
    options.words = true;
    options.output = Some(output_path.clone());
    options.strict = true;

    let config = ConfigQueryService::build(options).expect("config should build");
    run_with_config(config).expect("analysis should succeed");

    let json = read_json(&output_path);
    let files = json["files"].as_array().expect("files array exists");
    assert_eq!(files.len(), 3);

    let summary = &json["summary"];
    assert!(summary["lines"].as_u64().unwrap() > 0);
    assert!(summary["chars"].as_u64().unwrap() > 0);
    assert!(summary["words"].as_u64().is_some());
    assert_eq!(summary["files"].as_u64().unwrap(), 3);

    let by_ext = json["by"].as_array().expect("by array exists");
    assert_eq!(by_ext.len(), 1);
    let rows = by_ext[0]["rows"].as_array().unwrap();
    let exts: Vec<_> = rows.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(exts.contains(&"rs"));
    assert!(exts.contains(&"md"));
    assert!(exts.contains(&"toml"));
}

#[test]
fn strict_mode_behavior_on_error() {
    let temp = TempDir::new("strict_mode_error", "count_lines_integration");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let path = temp.write_file("no_read.txt", "secret");
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o000);
        std::fs::set_permissions(&path, perms).unwrap();
    }

    #[cfg(windows)]
    {
        temp.write_file("valid.txt", "content");
    }

    let mut options = base_options(temp.path());
    options.strict = true;

    let config = ConfigQueryService::build(options).expect("config builds");
    let result = run_with_config(config);

    #[cfg(unix)]
    assert!(result.is_err(), "strict mode should fail when unreadable files exist");

    #[cfg(windows)]
    assert!(result.is_ok(), "Windows fallback path should succeed");
}

#[test]
fn filtering_by_lines_range() {
    let temp = TempWorkspace::new("filter_lines", "count_lines_integration");
    temp.create_file("small.txt", "one\ntwo\n");
    temp.create_file("medium.txt", "1\n2\n3\n4\n5\n");
    temp.create_file("large.txt", "a\nb\nc\nd\ne\nf\ng\nh\ni\nj\n");

    let output_path = temp.path().join("output.json");

    let mut filters = FilterOptions::default();
    filters.min_lines = Some(3);
    filters.max_lines = Some(7);

    let mut options = base_options(temp.path());
    options.filters = filters;
    options.output = Some(output_path.clone());

    let config = ConfigQueryService::build(options).expect("config builds");
    run_with_config(config).expect("analysis succeeds");

    let json = read_json(&output_path);
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    let lines = files[0]["lines"].as_u64().unwrap();
    assert!((3..=7).contains(&lines));
}

#[test]
fn sorting_with_multiple_keys() {
    let temp = TempWorkspace::new("multi_sort", "count_lines_integration");
    temp.create_file("a_big.txt", "line1\nline2\nline3\nline4\nline5\n");
    temp.create_file("b_big.txt", "x1\nx2\nx3\nx4\nx5\n");
    temp.create_file("c_small.txt", "single line\n");

    let output_path = temp.path().join("sorted.json");

    let mut options = base_options(temp.path());
    options.sort_specs = vec![(SortKey::Lines, true), (SortKey::Name, false)];
    options.output = Some(output_path.clone());

    let config = ConfigQueryService::build(options).expect("config builds");
    run_with_config(config).expect("analysis succeeds");

    let json = read_json(&output_path);
    let files = json["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);

    let names: Vec<_> = files.iter().map(|f| f["file"].as_str().unwrap()).collect();
    assert!(names[0].contains("a_big"));
    assert!(names[1].contains("b_big"));
    assert!(names[2].contains("c_small"));
}

#[test]
fn test_with_builder_pattern() {
    let stats = vec![
        FileStatsBuilder::new("test.rs").lines(100).chars(500).words(75).build(),
        FileStatsBuilder::new("lib.rs").lines(200).chars(1000).words(150).build(),
    ];

    assert_stats(&stats[0]).has_lines(100).has_chars(500).has_words(Some(75)).has_ext("rs");

    assert_stats(&stats[1]).has_lines(200).has_chars(1000);
}

#[test]
#[ignore]
fn performance_large_file_set() {
    use std::time::Instant;

    let temp = TempWorkspace::new("perf_large", "count_lines_integration");
    for i in 0..100 {
        let content = format!("Line {}\n", i).repeat(10);
        temp.create_file(&format!("file_{:03}.txt", i), &content);
    }

    let mut options = base_options(temp.path());
    options.format = OutputFormat::Json;
    options.sort_specs = vec![(SortKey::Lines, true)];
    options.top_n = Some(10);
    options.output = None;

    let start = Instant::now();
    let config = ConfigQueryService::build(options).unwrap();
    run_with_config(config).unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 1, "processing 100 files took {:?}", elapsed);
}
