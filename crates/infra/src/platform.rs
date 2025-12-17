// crates/infra/src/platform.rs
//! Platform-specific abstractions for cross-platform compatibility.
//!
//! This module centralizes OS-specific logic to improve portability and reduce
//! scattered conditional compilation directives throughout the codebase.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

// ============================================================================
// Path Normalization
// ============================================================================

/// Trait for platform-aware path normalization used in sorting and deduplication.
pub trait PathNormalizer {
    /// Type of the normalized key used for comparison.
    type Key: Ord + Eq;

    /// Normalize a path to a comparable key.
    fn normalize(&self, path: &Path) -> Self::Key;
}

/// Windows path normalizer - case-insensitive comparison.
#[cfg(windows)]
pub struct WindowsPathNormalizer;

#[cfg(windows)]
impl PathNormalizer for WindowsPathNormalizer {
    type Key = String;

    fn normalize(&self, path: &Path) -> Self::Key {
        path.to_string_lossy().to_lowercase()
    }
}

/// Unix path normalizer - case-sensitive byte comparison.
#[cfg(unix)]
pub struct UnixPathNormalizer;

#[cfg(unix)]
impl PathNormalizer for UnixPathNormalizer {
    type Key = Vec<u8>;

    fn normalize(&self, path: &Path) -> Self::Key {
        use std::os::unix::ffi::OsStrExt;
        path.as_os_str().as_bytes().to_vec()
    }
}

/// Fallback path normalizer for other platforms.
#[cfg(all(not(windows), not(unix)))]
pub struct FallbackPathNormalizer;

#[cfg(all(not(windows), not(unix)))]
impl PathNormalizer for FallbackPathNormalizer {
    type Key = String;

    fn normalize(&self, path: &Path) -> Self::Key {
        path.to_string_lossy().into_owned()
    }
}

/// Default path normalizer for the current platform.
#[cfg(windows)]
pub type DefaultPathNormalizer = WindowsPathNormalizer;

#[cfg(unix)]
pub type DefaultPathNormalizer = UnixPathNormalizer;

#[cfg(all(not(windows), not(unix)))]
pub type DefaultPathNormalizer = FallbackPathNormalizer;

/// Create a default path normalizer for the current platform.
pub fn default_path_normalizer() -> DefaultPathNormalizer {
    #[cfg(windows)]
    return WindowsPathNormalizer;

    #[cfg(unix)]
    return UnixPathNormalizer;

    #[cfg(all(not(windows), not(unix)))]
    return FallbackPathNormalizer;
}

// ============================================================================
// Directory Loop Detection
// ============================================================================

/// Platform-aware directory loop detector for symlink traversal.
pub enum DirectoryLoopDetector {
    #[cfg(unix)]
    Inode(HashSet<(u64, u64)>),

    #[cfg(not(unix))]
    Canonical(HashSet<PathBuf>),
}

impl DirectoryLoopDetector {
    /// Create a new directory loop detector.
    pub fn new() -> Self {
        #[cfg(unix)]
        return Self::Inode(HashSet::new());

        #[cfg(not(unix))]
        return Self::Canonical(HashSet::new());
    }

    /// Check if a directory has been visited. Returns true if this is a new directory.
    /// On Unix, uses (dev, ino) pairs. On other platforms, uses canonical paths.
    pub fn visit(&mut self, path: &Path, _metadata: Option<&std::fs::Metadata>) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            if let Self::Inode(set) = self {
                if let Some(md) = _metadata {
                    let key = (md.dev(), md.ino());
                    return set.insert(key);
                } else if let Ok(md) = std::fs::metadata(path) {
                    let key = (md.dev(), md.ino());
                    return set.insert(key);
                }
            }
            false
        }

        #[cfg(not(unix))]
        {
            let Self::Canonical(set) = self;
            if let Ok(canon) = std::fs::canonicalize(path) {
                return set.insert(canon);
            }
            false
        }
    }
}

impl Default for DirectoryLoopDetector {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Cache Directory Resolution
// ============================================================================

/// Resolves platform-appropriate cache directory paths.
pub struct CacheDirectoryResolver;

impl CacheDirectoryResolver {
    /// Resolve the cache directory for the application.
    ///
    /// On Unix-like systems:
    /// 1. `$XDG_CACHE_HOME/count_lines` if XDG_CACHE_HOME is set
    /// 2. `$HOME/.cache/count_lines` if HOME is set
    /// 3. `.cache/count_lines` as fallback
    ///
    /// On Windows:
    /// 1. `%LOCALAPPDATA%\count_lines\cache` if LOCALAPPDATA is set
    /// 2. `%APPDATA%\count_lines\cache` if APPDATA is set
    /// 3. `.cache/count_lines` as fallback
    pub fn resolve() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            Self::resolve_windows()
        }

        #[cfg(not(windows))]
        {
            Self::resolve_unix()
        }
    }

    #[cfg(windows)]
    fn resolve_windows() -> Option<PathBuf> {
        use std::env;

        // Try LOCALAPPDATA first (preferred on Windows)
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            let mut dir = PathBuf::from(local_app_data);
            dir.push("count_lines");
            dir.push("cache");
            if Self::ensure_dir(&dir).is_ok() {
                return Some(dir);
            }
        }

        // Fall back to APPDATA
        if let Some(app_data) = env::var_os("APPDATA") {
            let mut dir = PathBuf::from(app_data);
            dir.push("count_lines");
            dir.push("cache");
            if Self::ensure_dir(&dir).is_ok() {
                return Some(dir);
            }
        }

        // Final fallback
        Self::resolve_fallback()
    }

    #[cfg(not(windows))]
    fn resolve_unix() -> Option<PathBuf> {
        use std::env;

        // Try XDG_CACHE_HOME first (XDG Base Directory specification)
        if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
            let mut dir = PathBuf::from(cache_home);
            dir.push("count_lines");
            if Self::ensure_dir(&dir).is_ok() {
                return Some(dir);
            }
        }

        // Fall back to HOME/.cache
        if let Some(home) = env::var_os("HOME") {
            let mut dir = PathBuf::from(home);
            dir.push(".cache/count_lines");
            if Self::ensure_dir(&dir).is_ok() {
                return Some(dir);
            }
        }

        // Final fallback
        Self::resolve_fallback()
    }

    fn resolve_fallback() -> Option<PathBuf> {
        use count_lines_shared_kernel::path::logical_absolute;

        let fallback = logical_absolute(Path::new(".cache/count_lines"));
        if Self::ensure_dir(&fallback).is_ok() {
            Some(fallback)
        } else {
            None
        }
    }

    fn ensure_dir(path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path)
    }
}

// ============================================================================
// Path Conversion
// ============================================================================

/// Trait for converting between byte arrays and paths.
pub trait PathConverter {
    /// Convert raw bytes to a PathBuf.
    fn from_bytes(bytes: &[u8]) -> PathBuf;
}

/// Unix path converter - preserves non-UTF-8 paths.
#[cfg(unix)]
pub struct UnixPathConverter;

#[cfg(unix)]
impl PathConverter for UnixPathConverter {
    fn from_bytes(bytes: &[u8]) -> PathBuf {
        use std::os::unix::ffi::OsStrExt;
        PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
    }
}

/// Windows path converter - assumes UTF-8 (git on Windows typically emits UTF-8).
#[cfg(windows)]
pub struct WindowsPathConverter;

#[cfg(windows)]
impl PathConverter for WindowsPathConverter {
    fn from_bytes(bytes: &[u8]) -> PathBuf {
        match std::str::from_utf8(bytes) {
            Ok(s) => PathBuf::from(s),
            Err(_) => PathBuf::from(String::from_utf8_lossy(bytes).into_owned()),
        }
    }
}

/// Fallback path converter.
#[cfg(all(not(unix), not(windows)))]
pub struct FallbackPathConverter;

#[cfg(all(not(unix), not(windows)))]
impl PathConverter for FallbackPathConverter {
    fn from_bytes(bytes: &[u8]) -> PathBuf {
        match std::str::from_utf8(bytes) {
            Ok(s) => PathBuf::from(s),
            Err(_) => PathBuf::from(String::from_utf8_lossy(bytes).into_owned()),
        }
    }
}

/// Default path converter for the current platform.
#[cfg(unix)]
pub type DefaultPathConverter = UnixPathConverter;

#[cfg(windows)]
pub type DefaultPathConverter = WindowsPathConverter;

#[cfg(all(not(unix), not(windows)))]
pub type DefaultPathConverter = FallbackPathConverter;

/// Convert bytes to a path using the platform-appropriate converter.
pub fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    DefaultPathConverter::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_normalizer_creates_comparable_keys() {
        let normalizer = default_path_normalizer();
        let path1 = Path::new("test/path");
        let path2 = Path::new("test/path");

        let key1 = normalizer.normalize(path1);
        let key2 = normalizer.normalize(path2);

        assert_eq!(key1, key2);
    }

    #[test]
    fn directory_loop_detector_tracks_visits() {
        let mut detector = DirectoryLoopDetector::new();
        let temp_dir = std::env::temp_dir();

        // First visit should succeed
        let visited = detector.visit(&temp_dir, None);
        assert!(visited);

        // Second visit should fail (already visited)
        let visited_again = detector.visit(&temp_dir, None);
        assert!(!visited_again);
    }

    #[test]
    fn cache_directory_resolver_returns_path() {
        let cache_dir = CacheDirectoryResolver::resolve();
        // Should return Some path on all platforms
        assert!(cache_dir.is_some());
    }

    #[test]
    fn path_from_bytes_handles_utf8() {
        let bytes = b"test/path.txt";
        let path = path_from_bytes(bytes);
        assert_eq!(path, PathBuf::from("test/path.txt"));
    }
}
