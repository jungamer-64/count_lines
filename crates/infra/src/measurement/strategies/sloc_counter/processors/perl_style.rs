// crates/infra/src/measurement/strategies/sloc_counter/processors/perl_style.rs
//! Perl言語のコメント処理
//!
//! Perl固有の対応:
//! - `#` 行コメント
//! - POD: `=pod`, `=head1` 等 ～ `=cut` (行頭必須)
//! - 文字列: `"..."`, `'...'` のみ考慮

use super::simple_hash_style::find_hash_outside_simple_string;

/// Perlプロセッサ
#[derive(Default)]
pub struct PerlProcessor {
    in_pod: bool,
    line_count: usize,
}

impl PerlProcessor {
    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // POD内の場合
        if self.in_pod {
            if line.starts_with("=cut") {
                self.in_pod = false;
            }
            return 0;
        }

        // POD開始判定
        if is_perl_pod_start(line) {
            self.in_pod = true;
            return 0;
        }

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

        // # より前にコードがあるか
        if let Some(hash_pos) = find_hash_outside_simple_string(line) {
            let before = &line[..hash_pos];
            if !before.trim().is_empty() {
                return 1;
            }
            return 0;
        }
        
        1
    }
}

/// Perl POD (Plain Old Documentation) の開始行かどうかを判定
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perl_processor_comment() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("# comment"), 0);
    }

    #[test]
    fn test_perl_processor_code() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("my $x = 1;"), 1);
    }

    #[test]
    fn test_perl_processor_pod() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("=head1 NAME"), 0);
        assert_eq!(p.process("Description"), 0);
        assert_eq!(p.process("=cut"), 0);
        assert_eq!(p.process("my $y = 2;"), 1);
    }

    #[test]
    fn test_perl_processor_shebang() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("#!/usr/bin/perl"), 0);
        assert_eq!(p.process("use strict;"), 1);
    }

    #[test]
    fn test_perl_processor_inline_comment() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("my $x = 1; # comment"), 1);
    }
}
