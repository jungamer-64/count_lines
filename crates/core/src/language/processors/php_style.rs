// src/language/processors/php_style.rs
//! PHP のコメント処理
//!
//! PHP は C系の `//, /* */` に加えて、Perl/Shell系の `#` 行コメントもサポートします。
//! ヒアドキュメント (Heredoc) `<<<` もサポートします。

use alloc::string::ToString;
use regex::Regex;

use super::super::heredoc_utils::HeredocContext;
use super::super::processor_trait::LineProcessor;
use super::super::string_utils::find_outside_string;

/// PHP プロセッサ
#[derive(Clone, Debug)]
pub struct PhpProcessor {
    in_block_comment: bool,
    heredoc_ctx: HeredocContext,
    heredoc_re: Regex,
}

impl LineProcessor for PhpProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment || self.heredoc_ctx.is_in_heredoc()
    }
}

impl Default for PhpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl PhpProcessor {
    /// Creates a new `PhpProcessor`.
    ///
    /// # Panics
    ///
    /// Panics if the internal regex pattern fails to compile (should never happen with hardcoded patterns).
    #[must_use]
    pub fn new() -> Self {
        Self {
            in_block_comment: false,
            heredoc_ctx: HeredocContext::new(),
            heredoc_re: Regex::new(r"<<<(?:([\w]+)|'([\w]+)'|\x22([\w]+)\x22)").unwrap(),
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    ///
    /// # Panics
    /// regexが不正な場合にパニックします（静的定義のため発生しません）。
    pub fn process(&mut self, line: &str) -> usize {
        // ヒアドキュメント処理
        if self.heredoc_ctx.is_in_heredoc() {
            // PHP end: `EOF;` or `EOF`
            let trimmed = line.trim();
            let check_target = trimmed.strip_suffix(';').unwrap_or(trimmed);

            if self.heredoc_ctx.check_end(check_target) {
                return 1;
            }

            if line.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        if self.in_block_comment {
            // ブロックコメント内
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                // 閉じた後にコードがあるかチェック
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty()
                    && !rest.trim().starts_with("//")
                    && !rest.trim().starts_with('#')
                {
                    // Check if heredoc start is in the rest?
                    return 1;
                }
            }
            return 0;
        }

        // Find comment start positions
        let block_start = find_outside_string(line, "/*");
        let line_slash = find_outside_string(line, "//");
        let line_hash = find_outside_string(line, "#");

        // Regex is in self.heredoc_re

        // Find earliest comment
        let first_comment = [block_start, line_slash, line_hash]
            .into_iter()
            .flatten()
            .min();

        // Check for heredoc start outside strings
        // And BEFORE any comment
        for caps in self.heredoc_re.captures_iter(line) {
            if let Some(matches) = caps.get(0) {
                let start = matches.start();
                if !is_inside_string(line, start) {
                    // Check if this heredoc start is before comments
                    if let Some(comment_pos) = first_comment
                        && start > comment_pos
                    {
                        continue;
                    }

                    let ident = caps
                        .get(1)
                        .or_else(|| caps.get(2))
                        .or_else(|| caps.get(3))
                        .unwrap()
                        .as_str()
                        .to_string();

                    // PHP 7.3+ supports indented heredoc content and closing marker.
                    self.heredoc_ctx.push(ident, true);
                }
            }
        }

        if let Some(pos) = first_comment {
            let before = &line[..pos];
            let has_code_before = !before.trim().is_empty();

            if block_start == Some(pos) {
                // Block comment start
                if let Some(end_offset) = line[pos + 2..].find("*/") {
                    // ends on same line
                    let after = &line[pos + 2 + end_offset + 2..];
                    if !after.trim().is_empty()
                        && !after.trim().starts_with("//")
                        && !after.trim().starts_with('#')
                    {
                        return 1;
                    }
                    return usize::from(has_code_before);
                }

                self.in_block_comment = true;
                return usize::from(has_code_before);
            }

            // Line comment
            return usize::from(has_code_before);
        }

        if line.trim().is_empty() {
            return 0;
        }

        1
    }

    pub fn reset(&mut self) {
        self.in_block_comment = false;
        self.heredoc_ctx.reset();
    }
}

fn is_inside_string(line: &str, target_pos: usize) -> bool {
    let bytes = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < target_pos && i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' {
            i += 2;
            continue;
        }
        if b == b'"' && !in_single {
            in_double = !in_double;
        } else if b == b'\'' && !in_double {
            in_single = !in_single;
        }
        i += 1;
    }
    in_single || in_double
}

// ============================================================================
// StatefulProcessor implementation
// ============================================================================

use super::super::processor_trait::StatefulProcessor;

/// State for `PhpProcessor`.
///
/// Note: Does not include `heredoc_re` (compiled regex).
#[derive(Debug, Clone, Default)]
pub struct PhpState {
    /// Whether currently inside a `/* */` block comment.
    pub in_block_comment: bool,
    /// Heredoc context (identifiers and `allow_indent` flags).
    pub heredoc_ctx: HeredocContext,
}

impl StatefulProcessor for PhpProcessor {
    type State = PhpState;

    fn get_state(&self) -> Self::State {
        PhpState {
            in_block_comment: self.in_block_comment,
            heredoc_ctx: self.heredoc_ctx.clone(),
        }
    }

    fn set_state(&mut self, state: Self::State) {
        self.in_block_comment = state.in_block_comment;
        self.heredoc_ctx = state.heredoc_ctx;
    }

    fn is_in_multiline_context(&self) -> bool {
        self.in_block_comment || self.heredoc_ctx.is_in_heredoc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_php_heredoc() {
        let mut p = PhpProcessor::new();
        assert_eq!(p.process("$x = <<<EOF"), 1);
        assert_eq!(p.process("Content"), 1);
        assert_eq!(p.process("EOF;"), 1);
    }

    #[test]
    fn test_php_nowdoc() {
        let mut p = PhpProcessor::new();
        assert_eq!(p.process("$x = <<<'EOF'"), 1);
        assert_eq!(p.process("Content"), 1);
        assert_eq!(p.process("EOF;"), 1);
    }
}
