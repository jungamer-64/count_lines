use count_lines_core::shared::patterns::parse_patterns;

#[test]
fn parses_valid_glob_patterns() {
    let patterns = vec!["*.rs".to_string(), "src/**".to_string()];
    let compiled = parse_patterns(&patterns).expect("patterns compile");
    assert_eq!(compiled.len(), 2);
    assert!(compiled[0].matches("lib.rs"));
    assert!(compiled[1].matches("src/main.rs"));
}

#[test]
fn returns_error_for_invalid_pattern() {
    let patterns = vec!["[[".to_string()];
    let err = parse_patterns(&patterns).expect_err("invalid pattern should error");
    assert!(err.to_string().contains("Invalid pattern"));
}
