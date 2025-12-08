// crates/infra/src/measurement/strategies/sloc_counter/processors/swift_style.rs
//! Swift言語のコメント処理
//!
//! Swift固有の対応:
//! - `//` 行コメント
//! - `/* */` ブロックコメント（ネスト対応）
//! - 拡張デリミタ文字列 `#"..."#`, `##"..."##` 等

use super::super::processor_trait::LineProcessor;
use super::super::string_utils::find_outside_string_swift;

/// Swift プロセッサ
pub struct SwiftProcessor {
    block_comment_depth: usize,
    in_block_comment: bool,
}

impl Default for SwiftProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor for SwiftProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment || self.block_comment_depth > 0
    }
}

impl SwiftProcessor {
    pub const fn new() -> Self {
        Self {
            block_comment_depth: 0,
            in_block_comment: false,
        }
    }

    pub fn process(&mut self, line: &str) -> usize {
        if self.block_comment_depth > 0 {
            self.process_nesting_block_line(line);
            return 0;
        }

        if let Some(line_comment_pos) = find_outside_string_swift(line, "//") {
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        if let Some(block_start) = find_outside_string_swift(line, "/*") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            self.block_comment_depth = 1;
            self.in_block_comment = true;
            let rest = &line[block_start + 2..];
            let rest_has_code = self.process_nesting_block_line(rest);
            if has_code_before || rest_has_code {
                return 1;
            }
            return 0;
        }

        1
    }

    fn process_nesting_block_line(&mut self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    self.block_comment_depth -= 1;
                    i += 2;
                    if self.block_comment_depth == 0 {
                        self.in_block_comment = false;
                        let rest = &line[i..];
                        if !rest.trim().is_empty() {
                            return self.process(rest) > 0;
                        }
                        return false;
                    }
                    continue;
                }
            }
            i += 1;
        }
        self.in_block_comment = self.block_comment_depth > 0;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swift_processor_line_comment() {
        let mut p = SwiftProcessor::new();
        assert_eq!(p.process("// comment"), 0);
        assert_eq!(p.process("let x = 1"), 1);
    }

    #[test]
    fn test_swift_processor_nested_block_comment() {
        let mut p = SwiftProcessor::new();
        assert_eq!(p.process("/* outer"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("/* nested */"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("let x = 1"), 1);
    }

    #[test]
    fn test_swift_processor_extended_delimiter() {
        let mut p = SwiftProcessor::new();
        assert_eq!(p.process(r##"let s = #"/* not a comment */"#"##), 1);
    }
}
