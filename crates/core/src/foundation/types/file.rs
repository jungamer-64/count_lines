use chrono::{DateTime, Local};
use std::path::PathBuf;

/// Metadata associated with a file entry.
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub is_text: bool,
    pub ext: String,
    pub name: String,
}

/// A path together with its metadata.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub meta: FileMeta,
}

/// Computed statistics for a file.
#[derive(Debug, Clone)]
pub struct FileStats {
    pub path: PathBuf,
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub ext: String,
    pub name: String,
}

impl FileStats {
    pub fn new(
        path: PathBuf,
        lines: usize,
        chars: usize,
        words: Option<usize>,
        meta: &FileMeta,
    ) -> Self {
        Self {
            path,
            lines,
            chars,
            words,
            size: meta.size,
            mtime: meta.mtime,
            ext: meta.ext.clone(),
            name: meta.name.clone(),
        }
    }
}
