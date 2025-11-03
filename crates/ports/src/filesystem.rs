// crates/ports/src/filesystem.rs
use std::path::PathBuf;

use chrono::{DateTime, Local};
use count_lines_shared_kernel::Result;
use serde::{Deserialize, Serialize};

/// Input parameters controlling file enumeration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(default)]
pub struct FileEnumerationPlan {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roots: Vec<PathBuf>,
    pub follow_links: bool,
    pub include_hidden: bool,
    pub no_default_prune: bool,
    pub fast_text_detect: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub include_patterns: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude_patterns: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub include_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude_dirs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exclude_dirs_only: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ext_filters: Vec<String>,
    pub size_range: (Option<u64>, Option<u64>),
    pub mtime_since: Option<DateTime<Local>>,
    pub mtime_until: Option<DateTime<Local>>,
    pub files_from: Option<PathBuf>,
    pub files_from0: Option<PathBuf>,
    pub use_git: bool,
    pub case_insensitive_dedup: bool,
    #[serde(default = "FileEnumerationPlan::default_respect_gitignore")]
    pub respect_gitignore: bool,
    pub use_ignore_overrides: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub overrides_include: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub overrides_exclude: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub force_text_exts: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub force_binary_exts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threads: Option<usize>,
}

impl FileEnumerationPlan {
    const fn default_respect_gitignore() -> bool {
        true
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for FileEnumerationPlan {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            follow_links: false,
            include_hidden: false,
            no_default_prune: false,
            fast_text_detect: false,
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
            exclude_dirs: Vec::new(),
            exclude_dirs_only: Vec::new(),
            ext_filters: Vec::new(),
            size_range: (None, None),
            mtime_since: None,
            mtime_until: None,
            files_from: None,
            files_from0: None,
            use_git: false,
            case_insensitive_dedup: false,
            respect_gitignore: true,
            use_ignore_overrides: false,
            overrides_include: Vec::new(),
            overrides_exclude: Vec::new(),
            force_text_exts: Vec::new(),
            force_binary_exts: Vec::new(),
            max_depth: None,
            threads: None,
        }
    }
}

/// DTO representing a file entry discovered by an input port.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(default)]
pub struct FileEntryDto {
    pub path: PathBuf,
    pub is_text: bool,
    pub size: u64,
    pub ext: String,
    pub name: String,
    pub mtime: Option<DateTime<Local>>,
}

impl FileEntryDto {
    #[must_use]
    pub fn new(
        path: PathBuf,
        is_text: bool,
        size: u64,
        ext: String,
        name: String,
        mtime: Option<DateTime<Local>>,
    ) -> Self {
        Self { path, is_text, size, ext, name, mtime }
    }
}

/// Port for enumerating file entries.
pub trait FileEnumerator: Send + Sync {
    fn collect(&self, plan: &FileEnumerationPlan) -> Result<Vec<FileEntryDto>>;
}
