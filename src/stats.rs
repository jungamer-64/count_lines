use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileStats {
    pub path: PathBuf,
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
    /// SLOC (Source Lines of Code) - 空行を除外した純粋コード行数
    #[serde(default)]
    pub sloc: Option<usize>,
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub ext: String,
    pub name: String,
    pub is_binary: bool,
}

impl FileStats {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        Self {
            path,
            lines: 0,
            chars: 0,
            words: None,
            sloc: None,
            size: 0,
            mtime: None,
            ext,
            name,
            is_binary: false,
        }
    }
}
