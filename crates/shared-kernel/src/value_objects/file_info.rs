// crates/shared-kernel/src/value_objects/file_info.rs
use std::{
    borrow::{Borrow, Cow},
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Wrapper around `PathBuf` that guarantees UTF-8 displayability in higher layers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FilePath(PathBuf);

impl FilePath {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }

    pub fn display(&self) -> std::path::Display<'_> {
        self.0.display()
    }

    /// Returns a UTF-8 view suitable for logging and UI; non UTF-8 segments are lossy converted.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }

    pub fn file_name(&self) -> Option<FileName> {
        self.0
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| FileName::new(s.to_string()))
    }

    pub fn extension(&self) -> Option<FileExtension> {
        self.0
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| FileExtension::new(s.to_lowercase()))
    }
}

impl From<PathBuf> for FilePath {
    fn from(path: PathBuf) -> Self {
        Self::new(path)
    }
}

impl From<&Path> for FilePath {
    fn from(path: &Path) -> Self {
        Self::new(path.to_path_buf())
    }
}
impl From<&str> for FilePath {
    fn from(path: &str) -> Self {
        Self::new(PathBuf::from(path))
    }
}
impl From<String> for FilePath {
    fn from(path: String) -> Self {
        Self::new(PathBuf::from(path))
    }
}

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
impl Deref for FilePath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Borrow<Path> for FilePath {
    fn borrow(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// UTF-8 file name captured during presentation; non UTF-8 names are filtered earlier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FileName(String);

impl FileName {
    #[must_use]
    pub fn new(name: String) -> Self {
        Self(name)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl From<String> for FileName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FileExtension(String);

impl FileExtension {
    /// Lowercased UTF-8 file extension; non UTF-8 values are dropped during harvesting.
    pub fn new(ext: String) -> Self {
        Self(ext.to_ascii_lowercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn no_ext() -> Self {
        Self(String::new())
    }
}

impl Default for FileExtension {
    fn default() -> Self {
        Self::no_ext()
    }
}

impl From<String> for FileExtension {
    fn from(ext: String) -> Self {
        Self::new(ext)
    }
}

impl From<&str> for FileExtension {
    fn from(ext: &str) -> Self {
        Self::new(ext.to_string())
    }
}

impl fmt::Display for FileExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            write!(f, "(noext)")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[must_use]
#[repr(transparent)]
#[serde(transparent)]
pub struct FileSize(u64);

impl FileSize {
    #[inline]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn bytes(self) -> u64 {
        self.0
    }

    /// Returns the size expressed in kibibytes (KiB).
    pub fn kilobytes(self) -> f64 {
        self.0 as f64 / 1024.0
    }

    /// Returns the size expressed in mebibytes (MiB).
    pub fn megabytes(self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0)
    }
}

impl From<u64> for FileSize {
    fn from(bytes: u64) -> Self {
        Self::new(bytes)
    }
}
impl From<FileSize> for u64 {
    fn from(size: FileSize) -> Self {
        size.bytes()
    }
}

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{}", self.to_human())
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl FileSize {
    /// Returns a base-2 human readable representation (KiB, MiB, GiB, TiB).
    pub fn to_human(self) -> String {
        const KIB: f64 = 1024.0;
        let bytes = self.bytes();
        if bytes < 1024 {
            return format!("{bytes} B");
        }

        let kib = bytes as f64 / KIB;
        if kib < KIB {
            return format!("{kib:.1} KiB");
        }

        let mib = kib / KIB;
        if mib < KIB {
            return format!("{mib:.1} MiB");
        }

        let gib = mib / KIB;
        if gib < KIB {
            return format!("{gib:.1} GiB");
        }

        let tib = gib / KIB;
        format!("{tib:.1} TiB")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[must_use]
#[repr(transparent)]
#[serde(transparent)]
pub struct ModificationTime(DateTime<Local>);

impl ModificationTime {
    pub fn new(timestamp: DateTime<Local>) -> Self {
        Self(timestamp)
    }

    pub fn timestamp(&self) -> &DateTime<Local> {
        &self.0
    }

    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }
}

impl From<DateTime<Local>> for ModificationTime {
    fn from(timestamp: DateTime<Local>) -> Self {
        Self::new(timestamp)
    }
}

impl fmt::Display for ModificationTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S"))
    }
}
