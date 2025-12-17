// src/language/processors/shell_style.rs
//! Shell言語のコメント・ヒアドキュメント処理
//!
//! シェルスクリプト (sh, bash, zsh) 固有の対応:
//! - `#` 行コメント
//! - ヒアドキュメント: `<<EOF`, `<<-EOF`, `<<'EOF'`, `<<"EOF"`
//!
//! ヒアドキュメント内の `#` はコメントとして扱われません。

use alloc::string::{String, ToString};
use regex::Regex;

use super::super::heredoc_utils::HeredocContext;
use super::super::processor_trait::LineProcessor;
use super::simple_hash_style::find_hash_outside_simple_string;

/// Shellプロセッサ
#[derive(Clone, Debug)]
pub struct ShellProcessor {
    heredoc_ctx: HeredocContext,
    line_count: usize,
    heredoc_re: Regex,
}

impl Default for ShellProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellProcessor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            heredoc_ctx: HeredocContext::default(),
            line_count: 0,
            heredoc_re: Regex::new(r"<<(-?)\s*(?:([^\s\x22'>|&;]+)|'([^']+)'|\x22([^\x22]+)\x22)")
                .unwrap(),
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        // ヒアドキュメント内かどうか
        if self.heredoc_ctx.is_in_heredoc() {
            // 終了判定
            if self.heredoc_ctx.check_end(line) {
                // 終了行自体はコードとみなすか？
                // 終了識別子の行はコードの一部 (構文) なので 1
                return 1;
            }
            // ヒアドキュメントの中身はデータ/コードとして扱う (コメントではない)
            // 空行は 0
            if line.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        // 行のトリム
        let trimmed = line.trim();

        // 1. Shebang check (first line)
        if trimmed.starts_with("#!") && self.line_count == 0 {
            self.line_count += 1;
            return 0;
        }
        self.line_count += 1;

        // 2. Comment check (# at start)
        if trimmed.starts_with('#') {
            return 0;
        }

        // 3. Scan for Heredoc start
        // ヒアドキュメント開始タグを探す
        // 注意: 文字列中やコメント中の << は無視する必要がある
        // 簡易実装として、# より前にある << を探す

        let hash_pos = find_hash_outside_simple_string(line);
        let check_limit = hash_pos.unwrap_or(line.len());
        let effective_line = &line[..check_limit];

        // ヒアドキュメント検出 (<<[-] ["']?WORD["']?)

        if let Some(captures) = self.find_heredoc_start(effective_line) {
            let allow_indent = captures.allow_indent;
            let ident = captures.ident;
            self.heredoc_ctx.push(ident, allow_indent);
        }

        // 4. Comment check (# inline)
        // 上記で hash_pos を計算済み
        if let Some(pos) = hash_pos {
            let before = &line[..pos];
            if !before.trim().is_empty() {
                return 1;
            }
            return 0;
        }

        // 空行チェック
        if trimmed.is_empty() {
            return 0;
        }

        1
    }
    fn find_heredoc_start(&self, line: &str) -> Option<HeredocStart> {
        // Regex pattern using alternation to avoid backreferences:
        // <<(-?)\s*(?:([^\s"'>|&;]+)|'([^']+)'|"([^"]+)")

        // 見つかった位置が文字列内かチェック
        for caps in self.heredoc_re.captures_iter(line) {
            if let Some(matches) = caps.get(0) {
                let start = matches.start();
                if !is_inside_string(line, start) {
                    let allow_indent = caps.get(1).is_some_and(|m| m.as_str() == "-");
                    let ident = caps
                        .get(2)
                        .or_else(|| caps.get(3))
                        .or_else(|| caps.get(4))
                        .unwrap()
                        .as_str()
                        .to_string();
                    return Some(HeredocStart {
                        ident,
                        allow_indent,
                    });
                }
            }
        }
        None
    }
}

impl LineProcessor for ShellProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn reset(&mut self) {
        self.heredoc_ctx.reset();
        self.line_count = 0;
    }

    fn is_in_block_comment(&self) -> bool {
        self.heredoc_ctx.is_in_heredoc()
    }
}

struct HeredocStart {
    ident: String,
    allow_indent: bool,
}

/// 指定位置が文字列 ("..." or '...') の中にあるか判定
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_heredoc_plain() {
        let mut p = ShellProcessor::new();
        assert_eq!(p.process("cat <<EOF"), 1);
        assert_eq!(p.process("  content"), 1);
        assert_eq!(p.process("# marked as comment but inside heredoc"), 1);
        assert_eq!(p.process("EOF"), 1);
        assert_eq!(p.process("  # real comment"), 0);
    }

    #[test]
    fn test_shell_heredoc_indented() {
        let mut p = ShellProcessor::new();
        assert_eq!(p.process("cat <<-EOF"), 1);
        assert_eq!(p.process("  content"), 1);
        assert_eq!(p.process("  EOF"), 1);
        assert_eq!(p.process("next command"), 1);
    }

    #[test]
    fn test_shell_heredoc_quoted() {
        let mut p = ShellProcessor::new();
        assert_eq!(p.process("cat <<'EOF'"), 1); // No expansion
        assert_eq!(p.process("  ${VAR}"), 1);
        assert_eq!(p.process("EOF"), 1);
    }

    #[test]
    fn test_shell_no_heredoc_in_string() {
        let mut p = ShellProcessor::new();
        // "<<" is inside string, so no heredoc start
        assert_eq!(p.process("echo '<<EOF'"), 1);
        assert_eq!(p.process("# comment"), 0); // should be comment
    }
}
