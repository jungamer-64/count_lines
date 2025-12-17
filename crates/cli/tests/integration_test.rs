//! Basic integration tests for the refactored `count_lines` crate

use count_lines_cli::stats::FileStats;
use count_lines_core::language::get_processor;
use hashbrown::HashMap;
use std::path::PathBuf;

/// Test that `get_processor` returns a working processor for various extensions
#[test]
fn test_get_processor_basic() {
    let map = HashMap::new();

    // Rust - Should handle SLOC
    let mut proc = get_processor("rs", &map);
    assert_eq!(proc.process_line("fn main() {}"), 1);
    assert_eq!(proc.process_line("// comment"), 0);

    // C - Should handle SLOC
    let mut proc = get_processor("c", &map);
    assert_eq!(proc.process_line("int x;"), 1);
    assert_eq!(proc.process_line("// comment"), 0);

    // Python - Should handle SLOC (assuming implementations are correct)
    let mut proc = get_processor("py", &map);
    assert_eq!(proc.process_line("def foo():"), 1);
    assert_eq!(proc.process_line("# comment"), 0);

    // Shell
    let mut proc = get_processor("sh", &map);
    assert_eq!(proc.process_line("echo hello"), 1);
    assert_eq!(proc.process_line("# comment"), 0);

    // Text - NoComment (empty map means default behavior?)
    // Actually "txt" maps to NoComment in get_processor default matching?
    // Wait, get_processor(ext) uses CommentStyle::from_extension.
    // CommentStyle("txt") -> None -> NoCommentProcessor.
    let mut proc = get_processor("txt", &map);
    assert_eq!(proc.process_line("text"), 1);
    assert_eq!(proc.process_line(""), 0);
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

/// Test reset functionality on a boxed processor
#[test]
fn test_line_processor_reset_boxed() {
    let map = HashMap::new();
    let mut proc = get_processor("rs", &map);

    // Process some lines (assume stateful processor like NestingCStyle)
    proc.process_line("/* start block");
    // Processor state is now in_block_comment

    // Reset should clear state
    proc.reset();

    // Should be fresh processor now (treats code as code)
    // If it was still in block comment, "fn test..." would be 0?
    // NestingCStyle: "fn test..." inside block is 0.
    // If reset works, it returns 1.
    assert_eq!(proc.process_line("fn test() {}"), 1);
}

/// Test `NoComment` usage
#[test]
fn test_no_comment_processor_basic() {
    let map = HashMap::new();
    let mut proc = get_processor("txt", &map);

    // NoComment just counts non-empty lines
    assert_eq!(proc.process_line("hello world"), 1);
    assert_eq!(proc.process_line(""), 0);
    assert_eq!(proc.process_line("   "), 0);
}
