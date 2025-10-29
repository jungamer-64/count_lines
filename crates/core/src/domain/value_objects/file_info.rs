use std::{
    fmt,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// ファイルパスを表す値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn file_name(&self) -> Option<FileName> {
        self.0.file_name().and_then(|s| s.to_str()).map(|s| FileName::new(s.to_string()))
    }

    pub fn extension(&self) -> Option<FileExtension> {
        self.0.extension().and_then(|s| s.to_str()).map(|s| FileExtension::new(s.to_lowercase()))
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

impl AsRef<Path> for FilePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// ファイル名を表す値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileName(String);

impl FileName {
    pub fn new(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

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

/// ファイル拡張子を表す値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileExtension(String);

impl FileExtension {
    pub fn new(ext: String) -> Self {
        Self(ext.to_lowercase())
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
        if self.0.is_empty() { write!(f, "(noext)") } else { write!(f, "{}", self.0) }
    }
}

/// ファイルサイズを表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    pub fn kilobytes(self) -> f64 {
        self.0 as f64 / 1024.0
    }

    pub fn megabytes(self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0)
    }
}

impl From<u64> for FileSize {
    fn from(bytes: u64) -> Self {
        Self::new(bytes)
    }
}

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ファイルの更新時刻を表す値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_path_operations() {
        let path = FilePath::new("/tmp/test.rs");
        assert_eq!(path.to_string(), "/tmp/test.rs");

        let name = path.file_name().unwrap();
        assert_eq!(name.as_str(), "test.rs");

        let ext = path.extension().unwrap();
        assert_eq!(ext.as_str(), "rs");
    }

    #[test]
    fn file_extension_display() {
        let ext = FileExtension::new("RS".to_string());
        assert_eq!(ext.as_str(), "rs");

        let no_ext = FileExtension::no_ext();
        assert_eq!(no_ext.to_string(), "(noext)");
    }

    #[test]
    fn file_size_conversions() {
        let size = FileSize::new(2048);
        assert_eq!(size.bytes(), 2048);
        assert_eq!(size.kilobytes(), 2.0);
    }
}
