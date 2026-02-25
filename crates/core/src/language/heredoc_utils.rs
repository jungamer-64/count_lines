// crates/core/src/language/heredoc_utils.rs
//! ヒアドキュメント処理ユーティリティ
//!
//! 複数行にわたる文字リテラル（ヒアドキュメント）の状態管理を提供します。

use alloc::string::String;
use alloc::vec::Vec;

/// ヒアドキュメントのエントリ情報
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeredocEntry {
    /// 終了識別子 (例: "EOF", "END")
    pub identifier: String,
    /// インデントを許可するか (例: <<-EOF, <<~EOF)
    pub allow_indent: bool,
}

/// ヒアドキュメントの状態管理
///
/// 複数のヒアドキュメントが1行に記述される場合（スタック）には今のところ対応していませんが、
/// 将来的な拡張のためにベクタで管理します。
/// 現状は先頭の要素のみを使用します。
#[derive(Debug, Default, Clone)]
pub struct HeredocContext {
    stack: Vec<HeredocEntry>,
}

impl HeredocContext {
    /// Creates a new empty `HeredocContext`.
    #[must_use]
    pub const fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// ヒアドキュメント内かどうか
    #[must_use]
    pub const fn is_in_heredoc(&self) -> bool {
        !self.stack.is_empty()
    }

    /// 新しいヒアドキュメントを開始
    pub fn push(&mut self, identifier: String, allow_indent: bool) {
        self.stack.push(HeredocEntry {
            identifier,
            allow_indent,
        });
    }

    /// 現在の行がヒアドキュメントの終了かどうかをチェックし、状態を更新する
    ///
    /// 終了した場合は true を返します。
    pub fn check_end(&mut self, line: &str) -> bool {
        if self.stack.is_empty() {
            return false;
        }

        let entry = &self.stack[0]; // 現在は単一レベルのみ簡易サポート

        let is_end = if entry.allow_indent {
            // インデント許可: トリムして比較
            line.trim() == entry.identifier
        } else {
            // インデント不許可: 行頭から完全一致 (改行は恐らく呼び出し元で処理済み)
            // ただし、入力lineには改行が含まれていない前提 (BufRead::read_line trim_end)
            // line が identifier と完全に一致するか
            line == entry.identifier || line.trim_end() == entry.identifier
        };

        if is_end {
            self.stack.remove(0);
            return true;
        }

        false
    }

    /// 強制リセット
    pub fn reset(&mut self) {
        self.stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_heredoc_context() {
        let mut ctx = HeredocContext::new();
        assert!(!ctx.is_in_heredoc());

        // <<-EOF form
        ctx.push("EOF".to_string(), true);
        assert!(ctx.is_in_heredoc());

        assert!(!ctx.check_end("  CONTENT"));
        assert!(ctx.check_end("  EOF")); // indents allowed
        assert!(!ctx.is_in_heredoc());

        // <<EOF form
        ctx.push("EOF".to_string(), false);
        assert!(!ctx.check_end("  EOF")); // should fail (indent not allowed)
        assert!(ctx.check_end("EOF")); // exact match
        assert!(!ctx.is_in_heredoc());
    }
}
