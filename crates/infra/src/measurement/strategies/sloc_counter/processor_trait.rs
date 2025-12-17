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

/// SLOC行処理トレイト
///
/// 各言語プロセッサはこのトレイトを実装することで、
/// 統一されたインターフェースを通じてSLOCカウントを行います。
pub trait LineProcessor: Default + Send {
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

    /// 処理状態をリセット
    ///
    /// 新しいファイルの処理を開始する前に呼び出します。
    fn reset(&mut self) {
        *self = Self::default();
    }

    /// 現在ブロックコメント内かどうかを返す（デバッグ用）
    ///
    /// すべてのプロセッサがブロックコメントをサポートしているわけではないため、
    /// デフォルトでは`false`を返します。
    fn is_in_block_comment(&self) -> bool {
        false
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
            if line.trim().starts_with("//") { 0 } else { 1 }
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
