//! Basic integration tests for the refactored `count_lines` crate

use count_lines::{language::SlocProcessor, stats::FileStats};
use std::path::PathBuf;

/// Test that `SlocProcessor` can be created from various extensions
#[test]
fn test_sloc_processor_from_extension() {
    // Rust - NestingCStyle
    let proc = SlocProcessor::from_extension("rs");
    assert!(matches!(proc, SlocProcessor::NestingCStyle(_)));

    // C - CStyle
    let proc = SlocProcessor::from_extension("c");
    assert!(matches!(proc, SlocProcessor::CStyle(_)));

    // Python
    let proc = SlocProcessor::from_extension("py");
    assert!(matches!(proc, SlocProcessor::Python(_)));

    // Ruby
    let proc = SlocProcessor::from_extension("rb");
    assert!(matches!(proc, SlocProcessor::Ruby(_)));

    // Shell - Shell (Specialized SimpleHash)
    let proc = SlocProcessor::from_extension("sh");
    assert!(matches!(proc, SlocProcessor::Shell(_)));

    // VHDL - SimplePrefix
    let proc = SlocProcessor::from_extension("vhd");
    assert!(matches!(proc, SlocProcessor::SimplePrefix(_)));

    // Text - NoComment
    let proc = SlocProcessor::from_extension("txt");
    assert!(matches!(proc, SlocProcessor::NoComment));
}

/// Test `FileStats` creation via `new()`
#[test]
fn test_file_stats_creation() {
    let stats = FileStats::new(PathBuf::from("test.rs"));

    assert_eq!(stats.path, PathBuf::from("test.rs"));
    assert_eq!(stats.lines, 0);
    assert_eq!(stats.sloc, None);
    assert_eq!(stats.ext, "rs");
    assert_eq!(stats.name, "test.rs");
    assert!(!stats.is_binary);
}

/// Test `FileStats` with fields set
#[test]
fn test_file_stats_with_values() {
    let mut stats = FileStats::new(PathBuf::from("example.py"));
    stats.lines = 100;
    stats.sloc = Some(80);
    stats.size = 2048;

    assert_eq!(stats.lines, 100);
    assert_eq!(stats.sloc, Some(80));
    assert_eq!(stats.size, 2048);
    assert_eq!(stats.ext, "py");
}

/// Test `SlocProcessor` default
#[test]
fn test_sloc_processor_default() {
    let proc = SlocProcessor::default();
    assert!(matches!(proc, SlocProcessor::NoComment));
}

/// Test `LineProcessor` trait on `SlocProcessor`
#[test]
fn test_line_processor_process_line() {
    use count_lines::language::LineProcessor;

    // Rust processor
    let mut proc = SlocProcessor::from_extension("rs");

    // Code line
    assert_eq!(proc.process_line("fn main() {}"), 1);

    // Comment line
    assert_eq!(proc.process_line("// comment"), 0);
}

/// Test reset functionality
#[test]
fn test_line_processor_reset() {
    use count_lines::language::LineProcessor;

    let mut proc = SlocProcessor::from_extension("rs");

    // Process some lines
    proc.process_line("/* start block");
    proc.process_line("still in block */");

    // Reset should clear state
    proc.reset();

    // Should be fresh processor now
    assert_eq!(proc.process_line("fn test() {}"), 1);
}

/// Test `NoComment` processor with empty lines
#[test]
fn test_no_comment_processor() {
    use count_lines::language::LineProcessor;

    let mut proc = SlocProcessor::from_extension("txt");

    // Non-empty line = code
    assert_eq!(proc.process_line("hello world"), 1);

    // Empty line = not code
    assert_eq!(proc.process_line(""), 0);

    // Whitespace only = not code
    assert_eq!(proc.process_line("   "), 0);
}
