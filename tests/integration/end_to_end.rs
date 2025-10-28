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
use serde_json::Value;
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
        let base = std::env::temp_dir().join("count_lines_integration");
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

fn write_file(root: &Path, name: &str, contents: &str) -> PathBuf {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, contents).unwrap();
    path
}

fn config_options(root: &Path, output: PathBuf) -> ConfigOptions {
    ConfigOptions {
        format: OutputFormat::Json,
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
        total_row: false,
        progress: false,
        ratio: false,
        output: Some(output),
        strict: true,
        compare: None,
    }
}

#[test]
fn end_to_end_generates_expected_json() {
    let temp = TempDir::new("end_to_end");
    write_file(
        &temp.path,
        "src/lib.rs",
        "fn main() {\n    println!(\"hello\");\n}\n",
    );
    write_file(&temp.path, "README.md", "# Count Lines\nMore text\n");

    let output_path = temp.path.join("result.json");
    let options = config_options(&temp.path, output_path.clone());
    let config = ConfigQueryService::build(options).expect("config builds");
    run_with_config(config).expect("run succeeds");

    let contents = fs::read_to_string(&output_path).expect("output exists");
    let json: Value = serde_json::from_str(&contents).expect("json parses");

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
