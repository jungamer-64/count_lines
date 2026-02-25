// crates/core/src/language/processors/matlab_style.rs
//! MATLAB/Octave言語のコメント処理
//!
//! MATLAB固有の対応:
//! - 行コメント: `%`
//! - ブロックコメント: `%{` ～ `%}`

use super::super::processor_trait::LineProcessor;

/// MATLAB プロセッサ
#[derive(Default)]
/// MATLAB SLOC processor.
#[derive(Debug)]
pub struct MatlabProcessor {
    in_block_comment: bool,
}

impl LineProcessor for MatlabProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl MatlabProcessor {
    #[must_use]
    /// Creates a new `MatlabProcessor`.
    pub const fn new() -> Self {
        Self {
            in_block_comment: false,
        }
    }

    /// Processes a line and returns the SLOC count.
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        if self.in_block_comment {
            if trimmed == "%}" {
                self.in_block_comment = false;
            }
            return 0;
        }

        if trimmed == "%{" {
            self.in_block_comment = true;
            return 0;
        }

        if trimmed.starts_with('%') {
            return 0;
        }

        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matlab_processor_line_comment() {
        let mut p = MatlabProcessor::new();
        assert_eq!(p.process("% comment"), 0);
        assert_eq!(p.process("x = 1;"), 1);
    }

    #[test]
    fn test_matlab_processor_block_comment() {
        let mut p = MatlabProcessor::new();
        assert_eq!(p.process("%{"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("comment"), 0);
        assert_eq!(p.process("%}"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("y = 2;"), 1);
    }
}
