// tests/common/helpers.rs
//! Test helper functions to reduce unwrap() calls and provide better error messages.

use std::fs;
use std::path::Path;

/// Create a test file with the given contents.
///
/// # Panics
///
/// Panics with a descriptive message if file creation fails.
pub fn create_test_file(path: &Path, contents: &[u8]) {
    fs::write(path, contents)
        .unwrap_or_else(|e| panic!("Failed to create test file at {:?}: {}", path, e));
}

/// Create all parent directories for the given path.
///
/// # Panics
///
/// Panics with a descriptive message if directory creation fails.
pub fn create_test_dir(path: &Path) {
    fs::create_dir_all(path)
        .unwrap_or_else(|e| panic!("Failed to create test directory at {:?}: {}", path, e));
}

/// Get file size in bytes.
///
/// # Panics
///
/// Panics with a descriptive message if metadata retrieval fails.
pub fn get_file_size(path: &Path) -> u64 {
    fs::metadata(path)
        .unwrap_or_else(|e| panic!("Failed to get metadata for {:?}: {}", path, e))
        .len()
}

/// Get file metadata.
///
/// # Panics
///
/// Panics with a descriptive message if metadata retrieval fails.
pub fn get_metadata(path: &Path) -> fs::Metadata {
    fs::metadata(path).unwrap_or_else(|e| panic!("Failed to get metadata for {:?}: {}", path, e))
}

/// Extract file name from path as a String.
///
/// # Panics
///
/// Panics with a descriptive message if the path has no file name.
pub fn get_file_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_else(|| panic!("Path {:?} has no file name", path))
        .to_string_lossy()
        .into()
}

/// Get a string value from a JSON object by key.
///
/// # Panics
///
/// Panics with a descriptive message if the key doesn't exist or value is not a string.
pub fn get_json_str<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_else(|| {
        panic!(
            "Expected string value for key '{}', got: {:?}",
            key, value[key]
        )
    })
}

/// Get a u64 value from a JSON object by key.
///
/// # Panics
///
/// Panics with a descriptive message if the key doesn't exist or value is not a u64.
pub fn get_json_u64(value: &serde_json::Value, key: &str) -> u64 {
    value[key].as_u64().unwrap_or_else(|| {
        panic!(
            "Expected u64 value for key '{}', got: {:?}",
            key, value[key]
        )
    })
}

/// Get an array value from a JSON object by key.
///
/// # Panics
///
/// Panics with a descriptive message if the key doesn't exist or value is not an array.
pub fn get_json_array<'a>(value: &'a serde_json::Value, key: &str) -> &'a Vec<serde_json::Value> {
    value[key].as_array().unwrap_or_else(|| {
        panic!(
            "Expected array value for key '{}', got: {:?}",
            key, value[key]
        )
    })
}

/// Lock a Mutex and return the guard.
///
/// # Panics
///
/// Panics with a descriptive message if the mutex is poisoned.
pub fn lock_mutex<T>(mutex: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|e| panic!("Failed to lock mutex: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn create_test_file_works() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.txt");
        create_test_file(&path, b"hello");
        assert_eq!(fs::read(&path).expect("read"), b"hello");
    }

    #[test]
    fn create_test_dir_works() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("a").join("b").join("c");
        create_test_dir(&path);
        assert!(path.exists());
        assert!(path.is_dir());
    }

    #[test]
    fn get_file_size_works() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("test.txt");
        create_test_file(&path, b"12345");
        assert_eq!(get_file_size(&path), 5);
    }

    #[test]
    fn get_file_name_works() {
        let path = Path::new("/foo/bar/test.txt");
        assert_eq!(get_file_name(path), "test.txt");
    }

    #[test]
    fn get_json_str_works() {
        let json = serde_json::json!({"name": "test"});
        assert_eq!(get_json_str(&json, "name"), "test");
    }

    #[test]
    fn get_json_u64_works() {
        let json = serde_json::json!({"count": 42});
        assert_eq!(get_json_u64(&json, "count"), 42);
    }

    #[test]
    fn get_json_array_works() {
        let json = serde_json::json!({"items": [1, 2, 3]});
        let array = get_json_array(&json, "items");
        assert_eq!(array.len(), 3);
    }

    #[test]
    fn lock_mutex_works() {
        let mutex = std::sync::Mutex::new(42);
        let guard = lock_mutex(&mutex);
        assert_eq!(*guard, 42);
    }
}
