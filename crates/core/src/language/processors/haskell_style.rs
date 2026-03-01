// crates/core/src/language/processors/haskell_style.rs
//! Haskell言語のコメント処理
//!
//! Haskell固有の対応:
//! - 行コメント: `--`
//! - ブロックコメント: `{-` ～ `-}` (ネスト対応)

use crate::language::processor_trait::LineProcessor;

/// Haskell プロセッサ
#[derive(Default)]
/// Haskell SLOC processor.
#[derive(Debug)]
pub struct HaskellProcessor {
    block_comment_depth: usize,
}

impl LineProcessor for HaskellProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.block_comment_depth > 0
    }
}

impl HaskellProcessor {
    #[must_use]
    /// Creates a new `HaskellProcessor`.
    pub const fn new() -> Self {
        Self {
            block_comment_depth: 0,
        }
    }

    /// Processes a line and returns the SLOC count.
    pub fn process(&mut self, line: &str) -> usize {
        if self.block_comment_depth > 0 {
            self.process_nesting_block(line);
            return 0;
        }

        if line.starts_with("--") {
            return 0;
        }

        if let Some(block_start) = line.find("{-") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");
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
                if bytes[i] == b'{' && bytes[i + 1] == b'-' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                if bytes[i] == b'-' && bytes[i + 1] == b'}' {
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

// ============================================================================
// StatefulProcessor implementation
// ============================================================================

use crate::language::processor_trait::StatefulProcessor;

/// State for `HaskellProcessor`.
#[derive(Debug, Clone, Default)]
pub struct HaskellState {
    /// Current nesting depth of block comments `{- -}`.
    pub block_comment_depth: usize,
}

impl StatefulProcessor for HaskellProcessor {
    type State = HaskellState;

    fn get_state(&self) -> Self::State {
        HaskellState {
            block_comment_depth: self.block_comment_depth,
        }
    }

    fn set_state(&mut self, state: Self::State) {
        self.block_comment_depth = state.block_comment_depth;
    }

    fn is_in_multiline_context(&self) -> bool {
        self.block_comment_depth > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_haskell_processor_line_comment() {
        let mut p = HaskellProcessor::new();
        assert_eq!(p.process("-- comment"), 0);
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_haskell_processor_nested_block() {
        let mut p = HaskellProcessor::new();
        assert_eq!(p.process("{- outer"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("{- nested -}"), 0);
        assert_eq!(p.process("-}"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("y = 2"), 1);
    }
}
