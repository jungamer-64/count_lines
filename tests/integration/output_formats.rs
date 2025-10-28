use count_lines_core::{
    run_with_config,
    ConfigOptions,
    ConfigQueryService,
    FilterOptions,
    domain::{
        grouping::ByMode,
        options::{OutputFormat, SortKey},
    },
};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let base = std::env::temp_dir().join("count_lines_output_formats");
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
        let path = base.join(format!("{prefix}_{unique}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn write_file(root: &Path, name: &str, contents: &str) {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn build_options(root: &Path, output: PathBuf, format: OutputFormat) -> ConfigOptions {
    ConfigOptions {
        format,
        sort_specs: vec![(SortKey::Lines, true)],
        top_n: None,
        by: vec![ByMode::Ext],
        summary_only: false,
        total_only: false,
        by_limit: None,
        filters: FilterOptions::default(),
        hidden: true,
        follow: false,
        use_git: false,
        jobs: Some(1),
        no_default_prune: true,
        abs_path: false,
        abs_canonical: false,
        trim_root: None,
        words: true,
        count_newlines_in_chars: false,
        text_only: false,
        fast_text_detect: false,
        files_from: None,
        files_from0: None,
        paths: vec![root.to_path_buf()],
        mtime_since: None,
        mtime_until: None,
        total_row: true,
        progress: false,
        ratio: false,
        output: Some(output),
        strict: true,
        compare: None,
    }
}

fn setup_fixture(temp: &TempDir) {
    write_file(
        &temp.path,
        "src/lib.rs",
        "fn main() {\n    println!(\"hello\");\n}\n",
    );
    write_file(&temp.path, "docs/readme.md", "# Intro\nLine\n");
}

#[test]
fn csv_output_contains_header_and_total_row() {
    let temp = TempDir::new("csv");
    setup_fixture(&temp);
    let output_path = temp.path.join("report.csv");
    let options = build_options(&temp.path, output_path.clone(), OutputFormat::Csv);
    let config = ConfigQueryService::build(options).expect("config builds");

    run_with_config(config).expect("run succeeds");
    let csv = fs::read_to_string(&output_path).expect("csv exists");

    assert!(csv.lines().next().unwrap().contains("lines,chars,words,file"));
    assert!(csv.contains("lib.rs"));
    assert!(csv.contains("TOTAL"));
}

#[test]
fn markdown_output_renders_table_and_group() {
    let temp = TempDir::new("markdown");
    setup_fixture(&temp);
    let output_path = temp.path.join("report.md");
    let options = build_options(&temp.path, output_path.clone(), OutputFormat::Md);
    let config = ConfigQueryService::build(options).expect("config builds");

    run_with_config(config).expect("run succeeds");
    let md = fs::read_to_string(&output_path).expect("markdown exists");

    assert!(md.starts_with("| LINES | CHARS | WORDS | FILE |"));
    assert!(md.contains("lib.rs"));
    assert!(md.contains("### By Extension"));
}
