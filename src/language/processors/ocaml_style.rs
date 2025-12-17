// src/language/processors/ocaml_style.rs
//! `OCaml`言語のコメント処理
//!
//! `OCaml`固有の対応:
//! - ブロックコメント: `(*` ～ `*)` (ネスト対応)
//! - 行コメントなし

use super::super::processor_trait::LineProcessor;

/// OCaml プロセッサ
#[derive(Default)]
pub struct OCamlProcessor {
    block_comment_depth: usize,
}

impl LineProcessor for OCamlProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.block_comment_depth > 0
    }
}

impl OCamlProcessor {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            block_comment_depth: 0,
        }
    }

    pub fn process(&mut self, line: &str) -> usize {
        if self.block_comment_depth > 0 {
            self.process_nesting_block(line);
            return 0;
        }

        if let Some(block_start) = line.find("(*") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            self.block_comment_depth = 1;
            let rest = &line[block_start + 2..];
            let rest_has_code = self.process_nesting_block(rest);
            return usize::from(has_code_before || rest_has_code);
        }

        1
    }

    fn process_nesting_block(&mut self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + 1 < bytes.len() {
                if bytes[i] == b'(' && bytes[i + 1] == b'*' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'*' && bytes[i + 1] == b')' {
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
    fn test_ocaml_processor_block_comment() {
        let mut p = OCamlProcessor::new();
        assert_eq!(p.process("(* comment *)"), 0);
        assert_eq!(p.process("let x = 1"), 1);
    }

    #[test]
    fn test_ocaml_processor_nested_block() {
        let mut p = OCamlProcessor::new();
        assert_eq!(p.process("(* outer"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("(* nested *)"), 0);
        assert_eq!(p.process("*)"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("let y = 2"), 1);
    }
}
