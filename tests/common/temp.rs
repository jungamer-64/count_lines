#![allow(dead_code)]
use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir as TempfileTempDir;

#[derive(Debug)]
pub struct TempDir {
    inner: TempfileTempDir,
}

impl TempDir {
    pub fn new(_prefix: &str, _namespace: &str) -> Self {
        // Use tempfile to create a secure unique temp directory
        let dir = tempfile::Builder::new()
            .prefix("count_lines_")
            .tempdir()
            .expect("create temp dir");
        Self { inner: dir }
    }

    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    pub fn write_file(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.path().join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        path
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct TempWorkspace {
    dir: TempDir,
}

impl TempWorkspace {
    #[allow(dead_code)]
    pub fn new(prefix: &str, namespace: &str) -> Self {
        Self {
            dir: TempDir::new(prefix, namespace),
        }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    #[allow(dead_code)]
    pub fn create_file(&self, rel: &str, contents: &str) -> PathBuf {
        self.dir.write_file(rel, contents)
    }
}
