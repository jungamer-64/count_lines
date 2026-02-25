// crates/core/src/language/processors/sql_style.rs
//! SQL言語のコメント処理
//!
//! SQL の -- 行コメントと /* */ ブロックコメントを処理します。

use super::super::processor_trait::LineProcessor;
use super::super::string_utils::find_outside_string_sql;

/// SQL SLOC processor.
///
/// - Line comments: `--` to end of line
/// - Block comments: `/* */`
/// - Ignores comment markers inside string literals (`'...'` and `"..."`)
#[derive(Debug)]
pub struct SqlProcessor {
    in_block_comment: bool,
}

impl Default for SqlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor for SqlProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl SqlProcessor {
    #[must_use]
    /// Creates a new `SqlProcessor`.
    pub const fn new() -> Self {
        Self {
            in_block_comment: false,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    /// Processes a line and returns the SLOC count.
    pub fn process(&mut self, line: &str) -> usize {
        if self.in_block_comment {
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() {
                    return self.process(rest);
                }
            }
            return 0;
        }

        // 行コメント (文字列外)
        if let Some(line_comment_pos) = find_outside_string_sql(line, "--") {
            let before = &line[..line_comment_pos];

            // -- より前にブロックコメント開始があるかチェック
            if let Some(block_start) = find_outside_string_sql(before, "/*") {
                return self.process_block_comment(line, block_start);
            }

            return usize::from(!before.trim().is_empty());
        }

        // ブロックコメント開始 (文字列外)
        if let Some(block_start) = find_outside_string_sql(line, "/*") {
            return self.process_block_comment(line, block_start);
        }

        1
    }

    fn process_block_comment(&mut self, line: &str, block_start: usize) -> usize {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();

        let after_start = &line[block_start + 2..];
        if let Some(end_offset) = after_start.find("*/") {
            let after = &after_start[end_offset + 2..];
            if has_code_before {
                return 1;
            } else if !after.trim().is_empty() {
                return self.process(after);
            }
            0
        } else {
            self.in_block_comment = true;
            usize::from(has_code_before)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_processor_line_comment() {
        let mut p = SqlProcessor::new();
        assert_eq!(p.process("-- comment"), 0);
    }

    #[test]
    fn test_sql_processor_code_with_line_comment() {
        let mut p = SqlProcessor::new();
        assert_eq!(p.process("SELECT * FROM t; -- comment"), 1);
    }

    #[test]
    fn test_sql_processor_string_with_block_comment_marker() {
        let mut p = SqlProcessor::new();
        assert_eq!(p.process("SELECT '/* not comment */' FROM users;"), 1);
    }

    #[test]
    fn test_sql_processor_string_with_line_comment_marker() {
        let mut p = SqlProcessor::new();
        assert_eq!(p.process("SELECT '-- not comment' FROM users;"), 1);
    }

    #[test]
    fn test_sql_processor_block_comment_multiline() {
        let mut p = SqlProcessor::new();
        assert_eq!(p.process("/*"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("  comment"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("SELECT 1;"), 1);
    }
}
