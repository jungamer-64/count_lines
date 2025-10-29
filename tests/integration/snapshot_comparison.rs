use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use count_lines_core::infrastructure::comparison;
use serde_json::json;

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(prefix: &str, contents: &str) -> Self {
        let base = std::env::temp_dir().join("count_lines_snapshot");
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string();
        let path = base.join(format!("{prefix}_{unique}.json"));
        fs::write(&path, contents).unwrap();
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
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
