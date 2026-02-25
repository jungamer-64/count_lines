// crates/core/src/language/processors/perl_style.rs
//! Perl言語のコメント処理
//!
//! Perl固有の対応:
//! - `#` 行コメント
//! - POD: `=pod`, `=head1` 等 ～ `=cut` (行頭必須)
//! - ヒアドキュメント: `<<EOF`, `<<'EOF'`, `<<"EOF"`

use alloc::string::ToString;
use regex::Regex;

use super::super::heredoc_utils::HeredocContext;
use super::super::processor_trait::LineProcessor;
use super::simple_hash_style::find_hash_outside_simple_string;

/// Perlプロセッサ
#[derive(Clone, Debug)]
pub struct PerlProcessor {
    in_pod: bool,
    line_count: usize,
    heredoc_ctx: HeredocContext,
    heredoc_re: Regex,
}

impl Default for PerlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor for PerlProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_pod || self.heredoc_ctx.is_in_heredoc()
    }
}



impl PerlProcessor {
    /// Creates a new `PerlProcessor`.
    ///
    /// # Panics
    ///
    /// Panics if the internal regex pattern fails to compile (should never happen with hardcoded patterns).
    #[must_use]
    pub fn new() -> Self {
        Self {
            in_pod: false,
            line_count: 0,
            heredoc_ctx: HeredocContext::new(),
            heredoc_re: Regex::new(r"<<\s*(?:([\w]+)|'([\w]+)'|\x22([\w]+)\x22)").unwrap(),
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    ///
    /// # Panics
    /// regexが不正な場合にパニックします（静的定義のため発生しません）。
    pub fn process(&mut self, line: &str) -> usize {
        // ヒアドキュメント処理
        if self.heredoc_ctx.is_in_heredoc() {
            if self.heredoc_ctx.check_end(line) {
                return 1;
            }
            if line.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        // POD内の場合
        if self.in_pod {
            if line.starts_with("=cut") {
                self.in_pod = false;
            }
            // POD内はコメント扱い (0)
            return 0;
        }

        // POD開始判定
        if is_perl_pod_start(line) {
            self.in_pod = true;
            return 0;
        }

        let trimmed = line.trim();

        // shebang行を除外 (最初の行のみ)
        if trimmed.starts_with("#!") && self.line_count == 0 {
            self.line_count += 1;
            return 0;
        }
        self.line_count += 1;

        // #で始まる行はコメント
        if trimmed.starts_with('#') {
            return 0;
        }

        // ヒアドキュメント開始検出
        // Perl: <<\s*(?:([\w]+)|'([\w]+)'|\x22([\w]+)\x22)

        for caps in self.heredoc_re.captures_iter(line) {
            if let Some(matches) = caps.get(0) {
                let start = matches.start();
                if !is_inside_string(line, start) {
                    // Group 1: unquoted, Group 2: single, Group 3: double
                    let ident = caps
                        .get(1)
                        .or_else(|| caps.get(2))
                        .or_else(|| caps.get(3))
                        .unwrap()
                        .as_str()
                        .to_string();
                    self.heredoc_ctx.push(ident, false);
                }
            }
        }

        // # より前にコードがあるか
        if let Some(hash_pos) = find_hash_outside_simple_string(line) {
            let before = &line[..hash_pos];
            if !before.trim().is_empty() {
                return 1;
            }
            return 0;
        }

        if trimmed.is_empty() {
            return 0;
        }

        1
    }

    /// Resets the processor state.
    pub fn reset(&mut self) {
        self.in_pod = false;
        self.line_count = 0;
        self.heredoc_ctx.reset();
    }
}

fn is_inside_string(line: &str, target_pos: usize) -> bool {
    // Same basic logic
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

/// Perl POD (Plain Old Documentation) の開始行かどうかを判定
#[must_use]
pub fn is_perl_pod_start(line: &str) -> bool {
    if !line.starts_with('=') {
        return false;
    }

    let bytes = line.as_bytes();
    if bytes.len() < 2 {
        return false;
    }

    let second = bytes[1];
    if !second.is_ascii_alphabetic() {
        return false;
    }

    line.starts_with("=pod")
        || line.starts_with("=head")
        || line.starts_with("=over")
        || line.starts_with("=item")
        || line.starts_with("=back")
        || line.starts_with("=encoding")
        || line.starts_with("=for")
        || line.starts_with("=begin")
}

// ============================================================================
// StatefulProcessor implementation
// ============================================================================

use super::super::processor_trait::StatefulProcessor;

/// State for `PerlProcessor`.
///
/// Note: Does not include `heredoc_re` (compiled regex).
#[derive(Debug, Clone, Default)]
pub struct PerlState {
    /// Whether currently inside POD documentation.
    pub in_pod: bool,
    /// Number of lines processed (for shebang detection).
    pub line_count: usize,
    /// Heredoc context (identifiers and `allow_indent` flags).
    pub heredoc_ctx: HeredocContext,
}

impl StatefulProcessor for PerlProcessor {
    type State = PerlState;

    fn get_state(&self) -> Self::State {
        PerlState {
            in_pod: self.in_pod,
            line_count: self.line_count,
            heredoc_ctx: self.heredoc_ctx.clone(),
        }
    }

    fn set_state(&mut self, state: Self::State) {
        self.in_pod = state.in_pod;
        self.line_count = state.line_count;
        self.heredoc_ctx = state.heredoc_ctx;
    }

    fn is_in_multiline_context(&self) -> bool {
        self.in_pod || self.heredoc_ctx.is_in_heredoc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perl_heredoc() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("print <<EOF;"), 1);
        assert_eq!(p.process("  content"), 1);
        assert_eq!(p.process("EOF"), 1);
    }

    #[test]
    fn test_perl_pod() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("=pod"), 0);
        assert_eq!(p.process("text"), 0);
        assert_eq!(p.process("=cut"), 0);
    }
}
