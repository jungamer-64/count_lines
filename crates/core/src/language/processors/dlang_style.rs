// crates/core/src/language/processors/dlang_style.rs
//! D言語のコメント処理
//!
//! D言語固有の対応:
//! - 行コメント: `//`
//! - ブロックコメント: `/* */`
//! - ネストブロックコメント: `/+ +/` (ネスト対応)

use super::super::processor_trait::LineProcessor;
use super::super::string_utils::find_outside_string;

/// D言語 プロセッサ
/// D language SLOC processor.
#[derive(Debug)]
pub struct DLangProcessor {
    block_comment_depth: usize,
    in_c_block: bool,
}

impl Default for DLangProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor for DLangProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.block_comment_depth > 0 || self.in_c_block
    }
}

impl DLangProcessor {
    #[must_use]
    /// Creates a new `DLangProcessor`.
    pub const fn new() -> Self {
        Self {
            block_comment_depth: 0,
            in_c_block: false,
        }
    }

    /// Processes a line and returns the SLOC count.
    pub fn process(&mut self, line: &str) -> usize {
        // ネストブロックコメント内
        if self.block_comment_depth > 0 {
            self.process_nesting_block(line);
            return 0;
        }

        // Cスタイルブロックコメント内
        if self.in_c_block {
            if let Some(pos) = line.find("*/") {
                self.in_c_block = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() {
                    return self.process(rest);
                }
            }
            return 0;
        }

        // 行コメント
        if let Some(pos) = find_outside_string(line, "//") {
            let before = &line[..pos];
            return usize::from(!before.trim().is_empty());
        }

        // ネストブロックコメント /+
        if let Some(pos) = find_outside_string(line, "/+") {
            let before = &line[..pos];
            let has_code_before = !before.trim().is_empty();
            self.block_comment_depth = 1;
            let rest = &line[pos + 2..];
            let rest_has_code = self.process_nesting_block(rest);
            return usize::from(has_code_before || rest_has_code);
        }

        // Cスタイルブロックコメント /*
        if let Some(pos) = find_outside_string(line, "/*") {
            let before = &line[..pos];
            let has_code_before = !before.trim().is_empty();
            let rest = &line[pos + 2..];
            if let Some(end_pos) = rest.find("*/") {
                let after = &rest[end_pos + 2..];
                if has_code_before || !after.trim().is_empty() {
                    return 1;
                }
                return 0;
            }
            self.in_c_block = true;
            return usize::from(has_code_before);
        }

        1
    }

    fn process_nesting_block(&mut self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() {
                if bytes[i] == b'/' && bytes[i + 1] == b'+' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'+' && bytes[i + 1] == b'/' {
                    self.block_comment_depth -= 1;
                    i += 2;
                    if self.block_comment_depth == 0 {
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
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dlang_processor_line_comment() {
        let mut p = DLangProcessor::new();
        assert_eq!(p.process("// comment"), 0);
        assert_eq!(p.process("int x = 1;"), 1);
    }

    #[test]
    fn test_dlang_processor_nested_block() {
        let mut p = DLangProcessor::new();
        assert_eq!(p.process("/+ outer"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("/+ nested +/"), 0);
        assert_eq!(p.process("+/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("int y = 2;"), 1);
    }
}
