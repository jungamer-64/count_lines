// crates/infra/src/measurement/strategies/sloc_counter/processor_trait.rs
//! SLOC行処理トレイト
//!
//! 各言語のコメント処理プロセッサに共通のインターフェースを提供します。
//!
//! # Example
//!
//! ```rust,ignore
//! use crate::measurement::strategies::sloc_counter::processor_trait::LineProcessor;
//!
//! struct MyProcessor { in_comment: bool }
//!
//! impl LineProcessor for MyProcessor {
//!     fn process_line(&mut self, line: &str) -> usize {
//!         if line.trim().starts_with("//") { 0 } else { 1 }
//!     }
//!
//!     fn reset(&mut self) {
//!         self.in_comment = false;
//!     }
//! }
//! ```

use alloc::boxed::Box;

/// 行統計情報
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LineStats {
    /// SLOCかどうか (0 or 1)
    pub sloc: usize,
    /// 文字数
    pub chars: usize,
    /// 単語数
    pub words: usize,
}

/// SLOC行処理トレイト
///
/// 各言語プロセッサはこのトレイトを実装することで、
/// 統一されたインターフェースを通じてSLOCカウントを行います。
pub trait LineProcessor: Send {
    /// 行を処理し、SLOCカウント (0 or 1) を返す
    ///
    /// # Arguments
    ///
    /// * `line` - 処理対象の行（改行を含まない）
    ///
    /// # Returns
    ///
    /// * `0` - コメントまたは空行（SLOCとしてカウントしない）
    /// * `1` - コード行（SLOCとしてカウント）
    fn process_line(&mut self, line: &str) -> usize;

    /// 行を処理し、完全な統計情報（SLOC, chars, words）を返す
    ///
    /// デフォルト実装では、`chars`と`words`のカウントを1パスで行い、
    /// `process_line`を呼び出してSLOCを判定します。
    fn process_line_stats(
        &mut self,
        line: &str,
        count_words: bool,
        count_newlines_in_chars: bool,
    ) -> LineStats {
        let sloc = self.process_line(line);

        let mut chars = 0;
        let mut words = 0;
        let mut in_word = false;

        // Perform single-pass scan for chars and words
        if count_newlines_in_chars {
            if count_words {
                for c in line.chars() {
                    chars += 1;
                    if c.is_whitespace() {
                        in_word = false;
                    } else if !in_word {
                        in_word = true;
                        words += 1;
                    }
                }
            } else {
                chars = line.chars().count();
            }
        } else {
            // Need to handle newline exclusion
            // We iterate chars but we must know if we are at the end to exclude newline chars from count
            // This is tricky in a single forward pass without lookahead or buffering,
            // but usually `line` comes from `read_until` so it ends with newline.

            // To be safe and correct with the existing logic "ends_with", we can just iterate.
            // But to exclude trailing newline from count, we can just subtract at the end.

            let mut total_chars = 0;
            if count_words {
                for c in line.chars() {
                    total_chars += 1;
                    if c.is_whitespace() {
                        in_word = false;
                    } else if !in_word {
                        in_word = true;
                        words += 1;
                    }
                }
            } else {
                total_chars = line.chars().count();
            }

            chars = total_chars;
            if line.ends_with("\r\n") {
                chars = chars.saturating_sub(2);
            } else if line.ends_with('\n') {
                chars = chars.saturating_sub(1);
            }
        }

        LineStats { sloc, chars, words }
    }

    /// 処理状態をリセット
    ///
    /// 新しいファイルの処理を開始する前に呼び出します。
    fn reset(&mut self) {
        // Default: no-op. Override if needed.
    }

    /// 現在ブロックコメント内かどうかを返す（デバッグ用）
    ///
    /// すべてのプロセッサがブロックコメントをサポートしているわけではないため、
    /// デフォルトでは`false`を返します。
    fn is_in_block_comment(&self) -> bool {
        false
    }
}

impl LineProcessor for Box<dyn LineProcessor> {
    fn process_line(&mut self, line: &str) -> usize {
        (**self).process_line(line)
    }

    fn process_line_stats(
        &mut self,
        line: &str,
        count_words: bool,
        count_newlines_in_chars: bool,
    ) -> LineStats {
        (**self).process_line_stats(line, count_words, count_newlines_in_chars)
    }

    fn reset(&mut self) {
        (**self).reset();
    }

    fn is_in_block_comment(&self) -> bool {
        (**self).is_in_block_comment()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestProcessor {
        count: usize,
    }

    impl LineProcessor for TestProcessor {
        fn process_line(&mut self, line: &str) -> usize {
            self.count += 1;
            usize::from(!line.trim().starts_with("//"))
        }

        fn reset(&mut self) {
            self.count = 0;
        }
    }

    #[test]
    fn test_line_processor_trait() {
        let mut proc = TestProcessor::default();
        assert_eq!(proc.process_line("// comment"), 0);
        assert_eq!(proc.process_line("code"), 1);
        assert_eq!(proc.count, 2);
    }

    #[test]
    fn test_reset() {
        let mut proc = TestProcessor { count: 5 };
        proc.reset();
        assert_eq!(proc.count, 0);
    }

    #[test]
    fn test_default_is_in_block_comment() {
        let proc = TestProcessor::default();
        assert!(!proc.is_in_block_comment());
    }
}
