//! テストフィクスチャ管理

use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// 一時ディレクトリ管理
#[allow(dead_code)]
pub struct TempWorkspace {
    root: PathBuf,
    files: Vec<PathBuf>,
}

#[allow(dead_code)]
impl TempWorkspace {
    pub fn new(prefix: &str) -> Self {
        let base = std::env::temp_dir().join("count_lines_test");
        fs::create_dir_all(&base).unwrap();

        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = base.join(format!("{prefix}_{unique}"));
        fs::create_dir(&root).unwrap();

        Self { root, files: Vec::new() }
    }

    /// ファイルを作成
    pub fn create_file(&mut self, path: &str, content: &str) -> &PathBuf {
        let full_path = self.root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        self.files.push(full_path);
        self.files.last().unwrap()
    }

    /// バイナリファイルを作成
    pub fn create_binary(&mut self, path: &str, content: &[u8]) -> &PathBuf {
        let full_path = self.root.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&full_path, content).unwrap();
        self.files.push(full_path);
        self.files.last().unwrap()
    }

    /// ディレクトリを作成
    pub fn create_dir(&mut self, path: &str) -> PathBuf {
        let full_path = self.root.join(path);
        fs::create_dir_all(&full_path).unwrap();
        full_path
    }

    /// ルートパスを取得
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// 作成されたすべてのファイルパスを取得
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// 標準的なRustプロジェクト構造を作成
    pub fn with_rust_project(mut self) -> Self {
        self.create_file(
            "Cargo.toml",
            r#"[package]
name = "test"
version = "0.1.0"
"#,
        );
        self.create_file("src/lib.rs", "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n");
        self.create_file("src/main.rs", "fn main() {\n    println!(\"Hello\");\n}\n");
        self.create_file("README.md", "# Test Project\n\nDescription\n");
        self
    }

    /// 多様なファイル種別を含むプロジェクト
    pub fn with_mixed_files(mut self) -> Self {
        self.create_file("code.rs", "fn test() {}\n");
        self.create_file("doc.md", "# Title\nContent\n");
        self.create_file("data.json", r#"{"key": "value"}"#);
        self.create_file("style.css", "body { color: red; }\n");
        self.create_binary("image.bin", &[0xFF, 0xD8, 0xFF, 0xE0]); // JPEG header
        self
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// 一時ファイル管理
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn new(prefix: &str, content: &str) -> Self {
        let base = std::env::temp_dir().join("count_lines_test");
        fs::create_dir_all(&base).unwrap();

        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = base.join(format!("{prefix}_{unique}.tmp"));
        fs::write(&path, content).unwrap();

        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_workspace_creates_and_cleans() {
        let workspace = TempWorkspace::new("test");
        assert!(workspace.path().exists());
        let path = workspace.path().to_path_buf();
        drop(workspace);
        assert!(!path.exists());
    }
}
