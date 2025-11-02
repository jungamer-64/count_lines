// crates/ports/src/filesystem.rs
use std::path::PathBuf;

use chrono::{DateTime, Local};
use count_lines_shared_kernel::Result;
use serde::{Deserialize, Serialize};

/// Input parameters controlling file enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEnumerationPlan {
    pub roots: Vec<PathBuf>,
    pub follow_links: bool,
    pub include_hidden: bool,
    pub no_default_prune: bool,
    pub fast_text_detect: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub ext_filters: Vec<String>,
    pub size_range: (Option<u64>, Option<u64>),
    pub mtime_since: Option<DateTime<Local>>,
    pub mtime_until: Option<DateTime<Local>>,
    pub files_from: Option<PathBuf>,
    pub files_from0: Option<PathBuf>,
    pub use_git: bool,
}

/// DTO representing a file entry discovered by an input port.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntryDto {
    pub path: PathBuf,
    pub is_text: bool,
    pub size: u64,
    pub ext: String,
    pub name: String,
    pub mtime: Option<DateTime<Local>>,
}

/// Port for enumerating file entries.
pub trait FileEnumerator: Send + Sync {
    fn collect(&self, plan: &FileEnumerationPlan) -> Result<Vec<FileEntryDto>>;
}
