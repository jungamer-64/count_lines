// src/language/processors/ruby_style.rs
//! Ruby言語のコメント処理
//!
//! Ruby固有の対応:
//! - `#` 行コメント
//! - 埋め込みドキュメント: `=begin` ～ `=end` (行頭必須)
//! - ヒアドキュメント: `<<EOF`, `<<-EOF`, `<<~EOF`
//! - 多重行文字列・変数の埋め込み (`#{...}`) 対応

use alloc::string::ToString;
use alloc::vec::Vec;
use regex::Regex;

use super::super::heredoc_utils::HeredocContext;
use super::super::processor_trait::LineProcessor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RubyScope {
    Interpolation, // #{ ... }
    String(u8),    // String with quote char (", ', `)
}

/// Rubyプロセッサ
#[derive(Clone, Debug)]
pub struct RubyProcessor {
    in_embedded_doc: bool,
    heredoc_ctx: HeredocContext,
    stack: Vec<RubyScope>,
    heredoc_re: Regex,
}

impl LineProcessor for RubyProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_embedded_doc || self.heredoc_ctx.is_in_heredoc() || self.is_in_string_scope()
    }
}

impl Default for RubyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl RubyProcessor {
    /// Creates a new `RubyProcessor`.
    ///
    /// # Panics
    ///
    /// Panics if the internal regex pattern fails to compile (should never happen with hardcoded patterns).
    #[must_use]
    pub fn new() -> Self {
        Self {
            in_embedded_doc: false,
            heredoc_ctx: HeredocContext::default(),
            stack: Vec::new(),
            heredoc_re: Regex::new(r"^<<([-~]?)(?:([\w]+)|'([\w]+)'|\x22([\w]+)\x22)").unwrap(),
        }
    }

    fn is_in_string_scope(&self) -> bool {
        matches!(self.stack.last(), Some(RubyScope::String(_)))
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    ///
    /// # Panics
    /// regexが不正な場合にパニックします（静的定義のため発生しません）。
    pub fn process(&mut self, line: &str) -> usize {
        // 1. ヒアドキュメント内容の処理 (最優先)
        if self.heredoc_ctx.is_in_heredoc() {
            if self.heredoc_ctx.check_end(line) {
                return 1;
            }
            if line.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        // 2. 埋め込みドキュメント (=begin ... =end)
        // 埋め込みドキュメントは通常、Code/Stringの外側で発生する
        if self.in_embedded_doc {
            if line.starts_with("=end") {
                self.in_embedded_doc = false;
            }
            return 0;
        }
        if self.stack.is_empty() && line.starts_with("=begin") {
            self.in_embedded_doc = true;
            return 0;
        }

        // 3. スキャンループ
        let mut has_code_token = false;
        let mut chars = line.char_indices().peekable();

        // Regex is now in self.heredoc_re

        while let Some((i, c)) = chars.next() {
            // エスケープ処理
            if c == '\\' {
                if chars.next().is_some() {
                    // エスケープされた文字は無条件にスキップ (ただしコードトークンとしてはカウント)
                    // 文字列内ならカウント、コメント内なら無視だが、コメント内かどうかが重要
                    if self.is_in_string_scope() || !c.is_whitespace() {
                        // 文字列内、または非空白なのでコードの一部かも
                        // Correct logic: we check state after processing char
                    }
                }
                // エスケープ自体も文字とみなす
                if self.is_in_string_scope() {
                    has_code_token = true;
                }
                continue;
            }

            // 現在のスコープ確認
            match self.stack.last() {
                Some(RubyScope::String(quote)) => {
                    if !c.is_whitespace() {
                        has_code_token = true;
                    }

                    let quote_char = *quote;
                    if c == quote_char as char {
                        // 文字列終了
                        self.stack.pop();
                    } else if (quote_char == b'"' || quote_char == b'`') && c == '#' {
                        // 埋め込み開始 check #{
                        if let Some((_, next_c)) = chars.peek()
                            && *next_c == '{'
                        {
                            chars.next(); // consume {
                            self.stack.push(RubyScope::Interpolation);
                        }
                    }
                }
                Some(RubyScope::Interpolation) | None => {
                    // Code mode
                    if c == '#' {
                        // コメント開始 (行末まで無視)
                        break;
                    }

                    if !c.is_whitespace() {
                        has_code_token = true;
                    }

                    // 文字列開始 check
                    if c == '"' || c == '\'' || c == '`' {
                        self.stack.push(RubyScope::String(c as u8));
                    }
                    // Interpolation終了 switch
                    else if c == '}'
                        && matches!(self.stack.last(), Some(RubyScope::Interpolation))
                    {
                        self.stack.pop();
                    } else if c == '<'
                        && let Some((_, next_c)) = chars.peek()
                        && *next_c == '<'
                    {
                        // "<<" Detect
                        if let Some(caps) = self.heredoc_re.captures(&line[i..]) {
                            let indent_flag = caps.get(1).map_or("", |m| m.as_str());
                            let allow_indent = indent_flag == "-" || indent_flag == "~";
                            let ident = caps
                                .get(2)
                                .or_else(|| caps.get(3))
                                .or_else(|| caps.get(4))
                                .unwrap()
                                .as_str()
                                .to_string();

                            self.heredoc_ctx.push(ident, allow_indent);

                            // マッチした分スキップするか？
                            // scannerはcharごとに進むので、Regexマッチ分を一気に飛ばさないと
                            // "<<" の次の "<" をまた処理してしまう可能性があるが、
                            // regexは "<<" を含むので、マッチしたらその分スキップすべき。
                            // しかし chars iterator を進めるのは難しい。
                            // 簡単のため、Heredoc開始は "Token" として扱い、これ以上解析しない？
                            // いや、同一行に続きがある場合がある: `foo(<<A, <<B)`
                            // なので、マッチした長さ分スキップしたい。

                            let match_len = caps.get(0).unwrap().len();
                            // skip match_len - 1 items (since we consumed '<')
                            // chars.nth(n) to skip n
                            if match_len > 1 {
                                // match_len includes starts at i.
                                // We consumed '<' at i.
                                // We need to skip match_len - 1 chars.
                                for _ in 0..(match_len - 1) {
                                    chars.next();
                                }
                            }
                            has_code_token = true;
                        }
                    }
                }
            }
        }

        // 行末終了時の処理
        // 何もしない (stackは維持される)

        usize::from(has_code_token)
    }

    pub fn reset(&mut self) {
        self.in_embedded_doc = false;
        self.heredoc_ctx.reset();
        self.stack.clear();
    }
}

// ============================================================================
// StatefulProcessor implementation
// ============================================================================

use super::super::processor_trait::StatefulProcessor;

/// State for `RubyProcessor`.
///
/// Note: Does not include `heredoc_re` as it's a compiled regex
/// that should not change during processing.
#[derive(Debug, Clone, Default)]
pub struct RubyState {
    /// Whether currently inside embedded documentation (=begin...=end).
    pub in_embedded_doc: bool,
    /// Heredoc context (identifiers and `allow_indent` flags).
    pub heredoc_ctx: HeredocContext,
    /// Current scope stack (strings, interpolations).
    pub stack: Vec<RubyScope>,
}

impl StatefulProcessor for RubyProcessor {
    type State = RubyState;

    fn get_state(&self) -> Self::State {
        RubyState {
            in_embedded_doc: self.in_embedded_doc,
            heredoc_ctx: self.heredoc_ctx.clone(),
            stack: self.stack.clone(),
        }
    }

    fn set_state(&mut self, state: Self::State) {
        self.in_embedded_doc = state.in_embedded_doc;
        self.heredoc_ctx = state.heredoc_ctx;
        self.stack = state.stack;
    }

    fn is_in_multiline_context(&self) -> bool {
        self.in_embedded_doc || self.heredoc_ctx.is_in_heredoc() || self.is_in_string_scope()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_heredoc() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("puts <<EOF"), 1);
        assert_eq!(p.process("  content"), 1);
        assert_eq!(p.process("EOF"), 1);
        assert_eq!(p.process("puts 'ok'"), 1);
    }

    #[test]
    fn test_ruby_heredoc_indented() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("puts <<-EOF"), 1);
        assert_eq!(p.process("  content"), 1);
        assert_eq!(p.process("  EOF"), 1);
    }

    #[test]
    fn test_ruby_heredoc_squiggly() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("puts <<~EOF"), 1);
        assert_eq!(p.process("  content"), 1);
        assert_eq!(p.process("  EOF"), 1);
    }

    #[test]
    fn test_ruby_stacked_heredoc() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("foo(<<A, <<B)"), 1);
        assert_eq!(p.process("content A"), 1);
        assert_eq!(p.process("A"), 1);
        assert_eq!(p.process("content B"), 1);
        assert_eq!(p.process("B"), 1);
    }

    #[test]
    fn test_ruby_multiline_string_content() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("x = \""), 1);
        assert_eq!(p.process("# string content"), 1);
        assert_eq!(p.process("\""), 1);
    }

    #[test]
    fn test_ruby_interpolation() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("x = \"#{"), 1);
        assert_eq!(p.process("  # comment"), 0);
        assert_eq!(p.process("  y = 1"), 1);
        assert_eq!(p.process("}\""), 1);
    }

    #[test]
    fn test_ruby_nested_interpolation() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("x = \"#{ \"nested #{ 1 }\" }\""), 1);
    }
}
