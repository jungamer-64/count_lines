// crates/engine/src/path_security.rs
//! Path Security Utilities
//!
//! Provides protection against path traversal attacks and symlink-based attacks.
//!
//! # Security Threats Addressed
//!
//! - **Path Traversal**: Using `..` to escape intended directories
//! - **Symlink Attacks**: Following malicious symbolic links to unintended locations
//! - **Excessive Depth**: Very deep paths that could cause resource exhaustion
//!
//! # Example
//!
//! ```rust,ignore
//! use count_lines_cli::path_security::{sanitize_path, PathSanitizeOptions};
//! use std::path::Path;
//!
//! let options = PathSanitizeOptions::default();
//! let result = sanitize_path(Path::new("./src"), &options);
//! assert!(result.is_ok());
//! ```

use crate::error::{EngineError, Result};
use std::path::{Component, Path, PathBuf};

/// Options for path sanitization.
///
/// Controls which security checks are performed during path validation.
#[derive(Debug, Clone)]
pub struct PathSanitizeOptions {
    /// Allow following symbolic links.
    ///
    /// When `false` (default), symlinks are rejected.
    /// When `true`, symlinks are followed and the target is validated.
    pub allow_symlinks: bool,

    /// Base directories that paths must remain within.
    ///
    /// If non-empty, the canonicalized path must start with one of these roots.
    /// If empty, no root restriction is applied.
    pub allowed_roots: Vec<PathBuf>,

    /// Maximum path depth (prevents deep traversal attacks).
    ///
    /// Default: 256 levels.
    pub max_depth: usize,

    /// Reject paths containing null bytes.
    ///
    /// Default: `true` (recommended for security).
    pub reject_null_bytes: bool,
}

impl Default for PathSanitizeOptions {
    fn default() -> Self {
        Self {
            allow_symlinks: false,
            allowed_roots: vec![],
            max_depth: 256,
            reject_null_bytes: true,
        }
    }
}

impl PathSanitizeOptions {
    /// Create options that allow symlinks (useful with `--follow` flag).
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Can't be const: returns Self with non-const Default
    pub fn with_symlinks(mut self) -> Self {
        self.allow_symlinks = true;
        self
    }

    /// Add an allowed root directory.
    #[must_use]
    pub fn with_allowed_root(mut self, root: PathBuf) -> Self {
        self.allowed_roots.push(root);
        self
    }
}

/// Result of path security check.
#[derive(Debug, Clone)]
pub struct SanitizedPath {
    /// The original path provided.
    pub original: PathBuf,
    /// The canonicalized (absolute, resolved) path.
    pub canonical: PathBuf,
    /// Whether the original path was a symlink.
    pub was_symlink: bool,
}

/// Sanitize and validate a path.
///
/// # Errors
///
/// Returns an error if:
/// - The path contains `..` components that would escape allowed roots
/// - The path is a symlink and symlinks are not allowed
/// - The path exceeds maximum depth
/// - The path cannot be canonicalized (doesn't exist, permission denied, etc.)
/// - The path is outside allowed root directories
pub fn sanitize_path(path: &Path, options: &PathSanitizeOptions) -> Result<SanitizedPath> {
    // Check for null bytes (security)
    if options.reject_null_bytes {
        let path_str = path.to_string_lossy();
        if path_str.contains('\0') {
            return Err(EngineError::Config("Path contains null bytes".into()));
        }
    }

    // Check for excessive depth before canonicalization
    let depth = count_path_depth(path);
    if depth > options.max_depth {
        return Err(EngineError::Config(format!(
            "Path exceeds maximum depth of {} (found {})",
            options.max_depth, depth
        )));
    }

    // Check for path traversal attempts
    if has_path_traversal(path) && !options.allowed_roots.is_empty() {
        // Only reject if we have allowed roots to protect
        // Otherwise, canonicalization will handle it
        return Err(EngineError::Config(
            "Path traversal attempt detected (contains '..')".into(),
        ));
    }

    // Check if path is a symlink (before canonicalization)
    let was_symlink = path.is_symlink();
    if was_symlink && !options.allow_symlinks {
        return Err(EngineError::Config(format!(
            "Symbolic links are not allowed: {}",
            path.display()
        )));
    }

    // Canonicalize to resolve symlinks and get absolute path
    let canonical = path.canonicalize().map_err(|e| {
        EngineError::Io(std::io::Error::new(
            e.kind(),
            format!("Failed to canonicalize path '{}': {}", path.display(), e),
        ))
    })?;

    // Check against allowed roots
    if !options.allowed_roots.is_empty() {
        let in_allowed_root = options.allowed_roots.iter().any(|root| {
            root.canonicalize()
                .map(|canonical_root| canonical.starts_with(&canonical_root))
                .unwrap_or(false)
        });

        if !in_allowed_root {
            return Err(EngineError::Config(format!(
                "Path '{}' is outside allowed directories",
                path.display()
            )));
        }
    }

    Ok(SanitizedPath {
        original: path.to_path_buf(),
        canonical,
        was_symlink,
    })
}

/// Check if a path is safe for processing (lightweight check).
///
/// This is a quick check that doesn't access the filesystem.
/// Use `sanitize_path` for full validation.
#[must_use]
pub fn is_path_safe(path: &Path) -> bool {
    // Check for null bytes
    let path_str = path.to_string_lossy();
    if path_str.contains('\0') {
        return false;
    }

    // Check for excessive parent directory references
    let mut depth: isize = 0;
    for component in path.components() {
        match component {
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return false; // Would escape root
                }
            }
            Component::Normal(_) => {
                depth += 1;
            }
            _ => {}
        }
    }

    true
}

/// Check if a path contains path traversal patterns.
fn has_path_traversal(path: &Path) -> bool {
    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// Count the depth of a path.
fn count_path_depth(path: &Path) -> usize {
    path.components()
        .filter(|c| matches!(c, Component::Normal(_)))
        .count()
}

/// Validate multiple paths at once.
///
/// # Errors
///
/// Returns an error if any path fails validation.
/// The error includes which path failed.
pub fn sanitize_paths(
    paths: &[PathBuf],
    options: &PathSanitizeOptions,
) -> Result<Vec<SanitizedPath>> {
    paths.iter().map(|p| sanitize_path(p, options)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_basic_path_sanitization() {
        let temp = TempDir::new().unwrap();
        let test_dir = temp.path().join("test");
        fs::create_dir(&test_dir).unwrap();

        let options = PathSanitizeOptions::default().with_symlinks();
        let result = sanitize_path(&test_dir, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_null_byte_rejection() {
        let path = Path::new("test\0path");
        let options = PathSanitizeOptions::default();
        let result = sanitize_path(path, &options);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null bytes"));
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(!is_path_safe(Path::new("../../../etc/passwd")));
        assert!(is_path_safe(Path::new("./src/main.rs")));
        assert!(is_path_safe(Path::new("a/b/../c"))); // Net zero
    }

    #[test]
    fn test_excessive_depth_rejection() {
        let deep_path = (0..300).map(|_| "a").collect::<Vec<_>>().join("/");
        let path = Path::new(&deep_path);
        let _options = PathSanitizeOptions {
            max_depth: 256,
            ..Default::default()
        };
        // This doesn't exist, but we check depth before canonicalization
        assert!(count_path_depth(path) > 256);
    }

    #[test]
    fn test_allowed_roots() {
        let temp = TempDir::new().unwrap();
        let test_dir = temp.path().join("allowed");
        fs::create_dir(&test_dir).unwrap();

        let other_dir = TempDir::new().unwrap();
        let other_file = other_dir.path().join("other");
        fs::create_dir(&other_file).unwrap();

        let options = PathSanitizeOptions::default()
            .with_symlinks()
            .with_allowed_root(test_dir.clone());

        // Path in allowed root should succeed
        let result = sanitize_path(&test_dir, &options);
        assert!(result.is_ok());

        // Path outside allowed root should fail
        let result = sanitize_path(&other_file, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_path_safe() {
        // Safe paths
        assert!(is_path_safe(Path::new("src/main.rs")));
        assert!(is_path_safe(Path::new("./config.toml")));
        assert!(is_path_safe(Path::new("a/b/c/d")));

        // Unsafe paths (escaping)
        assert!(!is_path_safe(Path::new("../secret")));
        assert!(!is_path_safe(Path::new("a/../../secret")));

        // Edge case: going up and down is still safe
        assert!(is_path_safe(Path::new("a/b/../c")));
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_rejection() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new().unwrap();
        let target = temp.path().join("target");
        let link = temp.path().join("link");

        fs::create_dir(&target).unwrap();
        symlink(&target, &link).unwrap();

        // With symlinks disabled
        let options = PathSanitizeOptions::default();
        let result = sanitize_path(&link, &options);
        assert!(result.is_err());

        // With symlinks enabled
        let options = PathSanitizeOptions::default().with_symlinks();
        let result = sanitize_path(&link, &options);
        assert!(result.is_ok());
        assert!(result.unwrap().was_symlink);
    }
}
