// crates/infra/src/measurement/strategies/sloc_counter/processors/c_style.rs
//! C系言語のコメント処理
//!
//! C/C++/Java/JavaScript/Rust/Go/Kotlin等の
//! `//` 行コメントと `/* */` ブロックコメントを処理します。

use super::super::string_utils::{find_outside_string_with_options, StringSkipOptions};

/// C系言語プロセッサ (//, /* */) - ネスト非対応
pub struct CStyleProcessor {
    options: StringSkipOptions,
    in_block_comment: bool,
}

impl CStyleProcessor {
    pub fn new(options: StringSkipOptions) -> Self {
        Self {
            options,
            in_block_comment: false,
        }
    }

    pub fn process(&mut self, line: &str) -> usize {
        if self.in_block_comment {
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                    return 1;
                }
            }
            return 0;
        }

        if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", &self.options) {
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        if let Some(block_start) = find_outside_string_with_options(line, "/*", &self.options) {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            
            if let Some(block_end) = line[block_start + 2..].find("*/") {
                let after = &line[block_start + 2 + block_end + 2..];
                let has_code_after = !after.trim().is_empty() 
                    && find_outside_string_with_options(after, "//", &self.options).is_none_or(|p| p > 0);
                if has_code_before || has_code_after {
                    return 1;
                }
                return 0;
            } else {
                self.in_block_comment = true;
                if has_code_before {
                    return 1;
                }
            }
            return 0;
        }

        1
    }

    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

/// C系言語プロセッサ - ネスト対応 (Rust, Kotlin, Scala)
pub struct NestingCStyleProcessor {
    options: StringSkipOptions,
    in_block_comment: bool,
    block_comment_depth: usize,
}

impl NestingCStyleProcessor {
    pub fn new(options: StringSkipOptions) -> Self {
        Self {
            options,
            in_block_comment: false,
            block_comment_depth: 0,
        }
    }

    pub fn process(&mut self, line: &str) -> usize {
        let mut count = 0;
        self.process_internal(line, &mut count);
        count
    }

    fn process_internal(&mut self, line: &str, count: &mut usize) {
        if self.block_comment_depth > 0 {
            self.process_nesting_block_line(line, count);
            return;
        }

        if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", &self.options) {
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                return;
            }
            *count += 1;
            return;
        }

        if let Some(block_start) = find_outside_string_with_options(line, "/*", &self.options) {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            self.block_comment_depth = 1;
            let rest = &line[block_start + 2..];
            self.process_nesting_block_line(rest, count);
            if has_code_before {
                *count += 1;
            }
            return;
        }

        *count += 1;
    }

    fn process_nesting_block_line(&mut self, line: &str, count: &mut usize) {
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
                            self.process_internal(rest, count);
                        }
                        return;
                    }
                    continue;
                }
            }
            i += 1;
        }
        self.in_block_comment = self.block_comment_depth > 0;
    }

    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment || self.block_comment_depth > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_style_processor_line_comment() {
        let mut p = CStyleProcessor::new(StringSkipOptions::default());
        assert_eq!(p.process("// comment"), 0);
        assert_eq!(p.process("int x = 1;"), 1);
    }

    #[test]
    fn test_c_style_processor_block_comment() {
        let mut p = CStyleProcessor::new(StringSkipOptions::default());
        assert_eq!(p.process("/* start"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("middle"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("int y = 2;"), 1);
    }

    #[test]
    fn test_c_style_processor_inline_comment() {
        let mut p = CStyleProcessor::new(StringSkipOptions::default());
        assert_eq!(p.process("int x = 1; // comment"), 1);
    }

    #[test]
    fn test_nesting_c_style_processor_nested_block() {
        let mut p = NestingCStyleProcessor::new(StringSkipOptions::default());
        assert_eq!(p.process("/* outer"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("/* nested */"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("let y = 2;"), 1);
    }
}
