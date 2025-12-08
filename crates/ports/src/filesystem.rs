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

    /// Create a builder for FileEnumerationPlan
    #[must_use]
    pub fn builder() -> FileEnumerationPlanBuilder {
        FileEnumerationPlanBuilder::default()
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

// ============================================================================
// Builder Pattern for FileEnumerationPlan
// ============================================================================

/// Builder for `FileEnumerationPlan`
///
/// Provides a fluent API for constructing `FileEnumerationPlan` instances.
///
/// # Example
///
/// ```rust,ignore
/// use count_lines_ports::FileEnumerationPlan;
///
/// let plan = FileEnumerationPlan::builder()
///     .roots(vec!["./src".into()])
///     .follow_links(true)
///     .include_patterns(vec!["*.rs".to_string()])
///     .max_depth(Some(5))
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct FileEnumerationPlanBuilder {
    inner: FileEnumerationPlan,
}

impl FileEnumerationPlanBuilder {
    /// Set the root directories to scan
    #[must_use]
    pub fn roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.inner.roots = roots;
        self
    }

    /// Add a single root directory
    #[must_use]
    pub fn root(mut self, root: PathBuf) -> Self {
        self.inner.roots.push(root);
        self
    }

    /// Set whether to follow symbolic links
    #[must_use]
    pub fn follow_links(mut self, follow: bool) -> Self {
        self.inner.follow_links = follow;
        self
    }

    /// Set whether to include hidden files
    #[must_use]
    pub fn include_hidden(mut self, include: bool) -> Self {
        self.inner.include_hidden = include;
        self
    }

    /// Set include patterns (glob)
    #[must_use]
    pub fn include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.inner.include_patterns = patterns;
        self
    }

    /// Set exclude patterns (glob)
    #[must_use]
    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.inner.exclude_patterns = patterns;
        self
    }

    /// Set extension filters
    #[must_use]
    pub fn ext_filters(mut self, exts: Vec<String>) -> Self {
        self.inner.ext_filters = exts;
        self
    }

    /// Set size range filter
    #[must_use]
    pub fn size_range(mut self, min: Option<u64>, max: Option<u64>) -> Self {
        self.inner.size_range = (min, max);
        self
    }

    /// Set maximum directory depth
    #[must_use]
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.inner.max_depth = depth;
        self
    }

    /// Set number of threads
    #[must_use]
    pub fn threads(mut self, threads: Option<usize>) -> Self {
        self.inner.threads = threads;
        self
    }

    /// Enable git mode (use git ls-files)
    #[must_use]
    pub fn use_git(mut self, use_git: bool) -> Self {
        self.inner.use_git = use_git;
        self
    }

    /// Set whether to respect .gitignore
    #[must_use]
    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.inner.respect_gitignore = respect;
        self
    }

    /// Set fast text detection mode
    #[must_use]
    pub fn fast_text_detect(mut self, fast: bool) -> Self {
        self.inner.fast_text_detect = fast;
        self
    }

    /// Build the FileEnumerationPlan
    #[must_use]
    pub fn build(self) -> FileEnumerationPlan {
        self.inner
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
