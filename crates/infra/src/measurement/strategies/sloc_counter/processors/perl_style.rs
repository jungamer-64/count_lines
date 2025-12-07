// crates/infra/src/measurement/strategies/sloc_counter/processors/perl_style.rs
//! Perl言語のコメント処理
//!
//! Perl固有の対応:
//! - `#` 行コメント
//! - POD: `=pod`, `=head1` 等 ～ `=cut` (行頭必須)
//! - 文字列: `"..."`, `'...'` のみ考慮

use super::simple_hash_style::find_hash_outside_simple_string;

// ============================================================================
// PerlProcessor
// ============================================================================

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
            // Perl: =cut で終了
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

        // # より前にコードがあるか (標準的な文字列のみ考慮)
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

// ============================================================================
// Helper functions
// ============================================================================

/// Perl POD (Plain Old Documentation) の開始行かどうかを判定
pub fn is_perl_pod_start(line: &str) -> bool {
    if !line.starts_with('=') {
        return false;
    }
    
    let bytes = line.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    
    // = の次が英字で始まる場合は POD コマンド
    let second = bytes[1];
    if !second.is_ascii_alphabetic() {
        return false;
    }
    
    // 主要な POD コマンドをチェック
    line.starts_with("=pod")
        || line.starts_with("=head")
        || line.starts_with("=over")
        || line.starts_with("=item")
        || line.starts_with("=back")
        || line.starts_with("=encoding")
        || line.starts_with("=for")
}

// ============================================================================
// 後方互換性のための関数
// ============================================================================

/// Perl スタイル (#) の処理 (後方互換)
pub fn process_perl_style(
    line: &str,
    in_embedded_doc: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // POD内の場合
    if *in_embedded_doc {
        // Perl: =cut で終了
        if line.starts_with("=cut") {
            *in_embedded_doc = false;
        }
        return;
    }

    // POD開始判定
    if is_perl_pod_start(line) {
        *in_embedded_doc = true;
        return;
    }

    // shebang行を除外
    if trimmed.starts_with("#!") && *count == 0 {
        return;
    }
    
    // #で始まる行はコメント
    if trimmed.starts_with('#') {
        return;
    }

    // # より前にコードがあるか (標準的な文字列のみ考慮)
    if let Some(hash_pos) = find_hash_outside_simple_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== PerlProcessor テスト ====================

    #[test]
    fn test_perl_processor_line_comment() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("# Perl comment"), 0);
    }

    #[test]
    fn test_perl_processor_code() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("my $x = 1;"), 1);
    }

    #[test]
    fn test_perl_processor_pod() {
        let mut p = PerlProcessor::default();
        assert_eq!(p.process("use strict;"), 1);
        assert_eq!(p.process("=pod"), 0);
        assert_eq!(p.process("This is POD documentation."), 0);
        assert_eq!(p.process("=cut"), 0);
        assert_eq!(p.process("print \"Hello\";"), 1);
    }

    // ==================== 後方互換関数テスト ====================

    #[test]
    fn test_perl_line_comment() {
        let mut in_doc = false;
        let mut count = 0;
        process_perl_style("# Perl comment", &mut in_doc, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_perl_line_comment_with_space() {
        let mut in_doc = false;
        let mut count = 0;
        process_perl_style("  # indented comment", &mut in_doc, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_perl_code() {
        let mut in_doc = false;
        let mut count = 0;
        process_perl_style("my $x = 1;", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_perl_code_with_inline_comment() {
        let mut in_doc = false;
        let mut count = 0;
        process_perl_style("my $x = 1;  # inline comment", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_perl_pod_basic() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("use strict;", &mut in_doc, &mut count);
        process_perl_style("=pod", &mut in_doc, &mut count);
        assert!(in_doc);
        process_perl_style("This is POD documentation.", &mut in_doc, &mut count);
        process_perl_style("=cut", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_perl_style("print \"Hello\";", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_perl_pod_with_head() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("my $x = 1;", &mut in_doc, &mut count);
        process_perl_style("=head1 NAME", &mut in_doc, &mut count);
        assert!(in_doc);
        process_perl_style("", &mut in_doc, &mut count);
        process_perl_style("MyModule - A sample module", &mut in_doc, &mut count);
        process_perl_style("", &mut in_doc, &mut count);
        process_perl_style("=head2 DESCRIPTION", &mut in_doc, &mut count);
        process_perl_style("", &mut in_doc, &mut count);
        process_perl_style("This module does something.", &mut in_doc, &mut count);
        process_perl_style("", &mut in_doc, &mut count);
        process_perl_style("=cut", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_perl_style("my $y = 2;", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_perl_pod_over_item() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("sub foo { 1 }", &mut in_doc, &mut count);
        process_perl_style("=over 4", &mut in_doc, &mut count);
        assert!(in_doc);
        process_perl_style("=item * First item", &mut in_doc, &mut count);
        process_perl_style("=item * Second item", &mut in_doc, &mut count);
        process_perl_style("=back", &mut in_doc, &mut count);
        process_perl_style("=cut", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_perl_style("sub bar { 2 }", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_perl_pod_head2_head3_head4() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("=head2 METHODS", &mut in_doc, &mut count);
        assert!(in_doc);
        process_perl_style("=head3 new()", &mut in_doc, &mut count);
        process_perl_style("=head4 Parameters", &mut in_doc, &mut count);
        process_perl_style("=cut", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_perl_style("sub new { }", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_perl_pod_encoding() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("=encoding utf8", &mut in_doc, &mut count);
        assert!(in_doc);
        process_perl_style("=cut", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_perl_style("print 1;", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_perl_pod_for() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("=for html <b>Bold</b>", &mut in_doc, &mut count);
        assert!(in_doc);
        process_perl_style("=cut", &mut in_doc, &mut count);
        assert!(!in_doc);
    }

    #[test]
    fn test_perl_shebang_not_counted() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("#!/usr/bin/perl", &mut in_doc, &mut count);
        process_perl_style("use warnings;", &mut in_doc, &mut count);
        process_perl_style("print 1;", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_perl_string_with_hash() {
        let mut in_doc = false;
        let mut count = 0;
        process_perl_style(r#"my $s = "hello # world";"#, &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_perl_single_quoted_string_with_hash() {
        let mut in_doc = false;
        let mut count = 0;
        process_perl_style("my $s = 'hello # world';", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_is_perl_pod_start_pod() {
        assert!(is_perl_pod_start("=pod"));
        assert!(is_perl_pod_start("=pod some text"));
    }

    #[test]
    fn test_is_perl_pod_start_head() {
        assert!(is_perl_pod_start("=head1 NAME"));
        assert!(is_perl_pod_start("=head2"));
        assert!(is_perl_pod_start("=head3 Methods"));
        assert!(is_perl_pod_start("=head4"));
    }

    #[test]
    fn test_is_perl_pod_start_over_item_back() {
        assert!(is_perl_pod_start("=over"));
        assert!(is_perl_pod_start("=over 4"));
        assert!(is_perl_pod_start("=item * bullet"));
        assert!(is_perl_pod_start("=back"));
    }

    #[test]
    fn test_is_perl_pod_start_encoding_for() {
        assert!(is_perl_pod_start("=encoding utf8"));
        assert!(is_perl_pod_start("=for html"));
    }

    #[test]
    fn test_is_perl_pod_start_not_pod() {
        assert!(!is_perl_pod_start("="));
        assert!(!is_perl_pod_start("= 1"));
        assert!(!is_perl_pod_start("=1"));
        assert!(!is_perl_pod_start("=="));
        assert!(!is_perl_pod_start("my $x = 1;"));
        assert!(!is_perl_pod_start("# comment"));
    }

    #[test]
    fn test_perl_subroutine() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("sub calculate {", &mut in_doc, &mut count);
        process_perl_style("    my ($a, $b) = @_;  # parameters", &mut in_doc, &mut count);
        process_perl_style("    return $a + $b;", &mut in_doc, &mut count);
        process_perl_style("}", &mut in_doc, &mut count);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_perl_regex_with_hash() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style(r#"my $re = qr/pattern/;"#, &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }
}
