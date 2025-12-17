use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use count_lines_core::infrastructure::comparison;
use serde_json::json;
use tempfile::{Builder as TempBuilder, TempDir as TempfileTempDir};

struct TempFile {
    _td: TempfileTempDir,
    path: PathBuf,
}

impl TempFile {
    fn new(prefix: &str, contents: &str) -> Self {
        let td = TempBuilder::new()
            .prefix(prefix)
            .tempdir()
            .expect("create tempdir");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = td.path().join(format!("{}_{}.json", prefix, unique));
        fs::write(&path, contents).unwrap();
        Self { _td: td, path }
    }
}

#[test]
fn snapshot_comparison_reports_differences() {
    let old_snapshot = json!({
        "files": [
            {"file": "src/lib.rs", "lines": 5, "chars": 50, "words": 9}
        ],
        "summary": {"lines": 5, "chars": 50, "words": 9, "files": 1}
    });
    let new_snapshot = json!({
        "files": [
            {"file": "src/lib.rs", "lines": 7, "chars": 70, "words": 11},
            {"file": "README.md", "lines": 3, "chars": 30, "words": 6}
        ],
        "summary": {"lines": 10, "chars": 100, "words": 17, "files": 2}
    });

    let old = TempFile::new("old", &old_snapshot.to_string());
    let new = TempFile::new("new", &new_snapshot.to_string());

    let diff = comparison::run(&old.path, &new.path).expect("comparison succeeds");
    assert!(diff.contains("Lines: 5 -> 10 (Δ 5)"));
    assert!(diff.contains("src/lib.rs"));
    assert!(diff.contains("(added)"));
}
