use std::path::Path;

use count_lines_core::shared::path::logical_absolute;

#[test]
fn returns_absolute_path_when_already_absolute() {
    let absolute = Path::new("/tmp/example");
    let result = logical_absolute(absolute);
    assert_eq!(result, absolute);
}

#[test]
fn prefixes_current_dir_for_relative_paths() {
    let relative = Path::new("src/lib.rs");
    let result = logical_absolute(relative);
    assert!(result.is_absolute());
    assert!(result.ends_with(relative));
}
