use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str, namespace: &str) -> Self {
        let base = std::env::temp_dir().join(namespace);
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string();
        let path = base.join(format!("{prefix}_{unique}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write_file(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.path.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, contents).unwrap();
        path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
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
        Self { dir: TempDir::new(prefix, namespace) }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    #[allow(dead_code)]
    pub fn create_file(&self, rel: &str, contents: &str) -> PathBuf {
        self.dir.write_file(rel, contents)
    }
}
