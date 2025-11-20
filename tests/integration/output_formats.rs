// tests/integration/output_formats.rs
use std::fs;

use count_lines_core::{
    application::ConfigQueryService,
    domain::{
        grouping::ByMode,
        options::{OutputFormat, SortKey},
    },
    run_with_config,
};

#[path = "../common/mod.rs"]
mod common;
use common::{ConfigOptionsBuilder, TempDir};

fn setup_fixture(temp: &TempDir) {
    temp.write_file("src/lib.rs", "fn main() {\n    println!(\"hello\");\n}\n");
    temp.write_file("docs/readme.md", "# Intro\nLine\n");
}

#[test]
fn csv_output_contains_header_and_total_row() {
    let temp = TempDir::new("csv", "count_lines_output_formats");
    setup_fixture(&temp);
    let output_path = temp.path().join("report.csv");
    let options = ConfigOptionsBuilder::new()
        .paths(vec![temp.path().to_path_buf()])
        .no_default_prune(true)
        .format(OutputFormat::Csv)
        .sort_specs(vec![(SortKey::Lines, true)])
        .by(vec![ByMode::Ext])
        .hidden(true)
        .words(true)
        .total_row(true)
        .output(output_path.clone())
        .strict(true)
        .build();
    let config = ConfigQueryService::build(options).expect("config builds");

    run_with_config(config).expect("run succeeds");
    let csv = fs::read_to_string(&output_path).expect("csv exists");

    assert!(csv.lines().next().unwrap().contains("lines,chars,words,file"));
    assert!(csv.contains("lib.rs"));
    assert!(csv.contains("TOTAL"));
}

#[test]
fn markdown_output_renders_table_and_group() {
    let temp = TempDir::new("markdown", "count_lines_output_formats");
    setup_fixture(&temp);
    let output_path = temp.path().join("report.md");
    let options = ConfigOptionsBuilder::new()
        .paths(vec![temp.path().to_path_buf()])
        .no_default_prune(true)
        .format(OutputFormat::Md)
        .sort_specs(vec![(SortKey::Lines, true)])
        .by(vec![ByMode::Ext])
        .hidden(true)
        .words(true)
        .total_row(true)
        .output(output_path.clone())
        .strict(true)
        .build();
    let config = ConfigQueryService::build(options).expect("config builds");

    run_with_config(config).expect("run succeeds");
    let md = fs::read_to_string(&output_path).expect("markdown exists");

    assert!(md.starts_with("| LINES | CHARS | WORDS | FILE |"));
    assert!(md.contains("lib.rs"));
    assert!(md.contains("### By Extension"));
}
