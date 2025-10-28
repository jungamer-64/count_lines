use count_lines_core::domain::config::{Filters, Range, SizeRange};

#[test]
fn range_contains_respects_optional_bounds() {
    let unconstrained = Range::default();
    assert!(unconstrained.contains(42));

    let lower = Range::new(Some(10), None);
    assert!(lower.contains(10));
    assert!(!lower.contains(9));

    let upper = Range::new(None, Some(5));
    assert!(upper.contains(5));
    assert!(!upper.contains(6));

    let bounded = Range::new(Some(3), Some(7));
    assert!(bounded.contains(3));
    assert!(bounded.contains(7));
    assert!(!bounded.contains(2));
    assert!(!bounded.contains(8));
}

#[test]
fn size_range_contains_respects_bounds() {
    let bounded = SizeRange::new(Some(1024), Some(4096));
    assert!(bounded.contains(2048));
    assert!(!bounded.contains(512));
    assert!(!bounded.contains(8192));

    let lower = SizeRange::new(Some(1024), None);
    assert!(lower.contains(4096));
    assert!(!lower.contains(512));
}

#[test]
fn filters_default_is_empty() {
    let filters = Filters::default();
    assert!(filters.include_patterns.is_empty());
    assert!(filters.exclude_patterns.is_empty());
    assert!(filters.include_paths.is_empty());
    assert!(filters.exclude_paths.is_empty());
    assert!(filters.exclude_dirs.is_empty());
    assert!(filters.ext_filters.is_empty());
    assert!(filters.filter_ast.is_none());
    assert!(filters.size_range.contains(0));
    assert!(filters.lines_range.contains(0));
    assert!(filters.chars_range.contains(0));
    assert!(filters.words_range.contains(0));
}
