// crates/infra/src/measurement/strategies/sloc_counter/processors/hash_style.rs
//! Hash系言語のコメント処理
//!
//! Ruby/Perl/Shell等の `#` コメントを処理します。
//! - Ruby: 埋め込みドキュメント（`=begin` ～ `=end`）
//! - Perl: POD（`=pod` / `=head1` 等 ～ `=cut`）
//! - Shell/YAML/Config: 単純な # コメント（複雑な文字列処理不要）
//!
//! Note: Python は複雑な文字列処理（Docstring, f-string）があるため
//! python_style.rs に分離されています。

/// Ruby スタイル (#) の処理
/// 
/// Ruby固有の対応:
/// - 埋め込みドキュメント: `=begin` ～ `=end` (行頭必須)
/// - 文字列: `"..."`, `'...'` のみ考慮
pub fn process_ruby_style(
    line: &str,
    in_embedded_doc: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // 埋め込みドキュメント内の場合
    if *in_embedded_doc {
        // Ruby: =end で終了
        if line.starts_with("=end") {
            *in_embedded_doc = false;
        }
        return;
    }

    // 埋め込みドキュメント開始判定（行頭の =begin で始まる）
    if line.starts_with("=begin") {
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

/// Perl スタイル (#) の処理
/// 
/// Perl固有の対応:
/// - POD: `=pod`, `=head1` 等 ～ `=cut` (行頭必須)
/// - 文字列: `"..."`, `'...'` のみ考慮
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

/// 単純な Hash スタイル (#) の処理
/// 
/// 対象: Shell, YAML, TOML, Dockerfile, Makefile, Config系など
/// 
/// 特徴:
/// - 複雑な文字列処理不要
/// - `"..."` と `'...'` のみ考慮（バッククォートや三重クォートなし）
/// - Docstringや埋め込みドキュメントなし
/// - 高速かつ安全な処理
pub fn process_simple_hash_style(
    line: &str,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // shebang行を除外
    if trimmed.starts_with("#!") && *count == 0 {
        return;
    }
    
    // #で始まる行はコメント
    if trimmed.starts_with('#') {
        return;
    }

    // # より前にコードがあるか (単純な文字列のみ考慮)
    if let Some(hash_pos) = find_hash_outside_simple_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

/// Perl POD (Plain Old Documentation) の開始行かどうかを判定
/// 
/// PODは `=` で始まり、英字が続くコマンドで開始される:
/// - `=pod`, `=head1`, `=head2`, `=head3`, `=head4`
/// - `=over`, `=item`, `=back`
/// - `=encoding`, `=for`, `=begin`, `=end`
fn is_perl_pod_start(line: &str) -> bool {
    if !line.starts_with('=') {
        return false;
    }
    
    let bytes = line.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    
    // = の次が英字で始まる場合は POD コマンド
    // (=begin は Ruby と共通なので上で処理済み、ここでは =pod, =head 等を検出)
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

/// 単純な文字列 ("..." / '...') 外で # を検索
/// 
/// Shell/YAML/Config等向けの軽量版。
/// Python の f-string や三重クォートは考慮しない。
fn find_hash_outside_simple_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // ダブルクォート文字列: "..."
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2; // エスケープシーケンスをスキップ
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // シングルクォート文字列: '...'
        if bytes[i] == b'\'' {
            i += 1;
            while i < bytes.len() {
                // シングルクォート内はエスケープなし (シェル的解釈)
                // ただし '' で1つの ' を表す場合があるので、次の文字もチェック
                if bytes[i] == b'\'' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        if bytes[i] == b'#' {
            return Some(i);
        }
        
        i += 1;
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Ruby テスト ====================

    #[test]
    fn test_ruby_line_comment() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style("# comment", &mut in_doc, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_ruby_code() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style("x = 1", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ruby_embedded_document() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("x = 1", &mut in_doc, &mut count);
        process_ruby_style("=begin", &mut in_doc, &mut count);
        assert!(in_doc);
        process_ruby_style("This is embedded documentation.", &mut in_doc, &mut count);
        process_ruby_style("It can span multiple lines.", &mut in_doc, &mut count);
        process_ruby_style("=end", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_ruby_style("y = 2", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ruby_embedded_document_with_comments() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("# regular comment", &mut in_doc, &mut count);
        process_ruby_style("def foo", &mut in_doc, &mut count);
        process_ruby_style("=begin", &mut in_doc, &mut count);
        process_ruby_style("  embedded doc", &mut in_doc, &mut count);
        process_ruby_style("=end", &mut in_doc, &mut count);
        process_ruby_style("  puts 'hello'", &mut in_doc, &mut count);
        process_ruby_style("end", &mut in_doc, &mut count);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_ruby_embedded_doc_must_start_at_line_beginning() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("x = 1  # =begin is not at line start", &mut in_doc, &mut count);
        process_ruby_style("y = 2", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Perl テスト ====================

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
    fn test_perl_shebang_not_counted() {
        let mut in_doc = false;
        let mut count = 0;

        process_perl_style("#!/usr/bin/perl", &mut in_doc, &mut count);
        process_perl_style("use warnings;", &mut in_doc, &mut count);
        process_perl_style("print 1;", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Simple Hash Style テスト ====================

    #[test]
    fn test_simple_hash_line_comment() {
        let mut count = 0;
        process_simple_hash_style("# comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_simple_hash_code() {
        let mut count = 0;
        process_simple_hash_style("x = 1", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_simple_hash_code_with_inline_comment() {
        let mut count = 0;
        process_simple_hash_style("x = 1  # comment", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_simple_hash_string_with_hash() {
        let mut count = 0;
        process_simple_hash_style(r#"s = "hello # world""#, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== INI/Config テスト ====================

    #[test]
    fn test_ini_hash_comment() {
        let mut count = 0;
        process_simple_hash_style("# INI comment", &mut count);
        process_simple_hash_style("[section]", &mut count);
        process_simple_hash_style("key = value", &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_conf_file() {
        let mut count = 0;
        process_simple_hash_style("# Configuration", &mut count);
        process_simple_hash_style("server = localhost", &mut count);
        process_simple_hash_style("port = 8080", &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_properties_file() {
        let mut count = 0;
        process_simple_hash_style("# Java properties", &mut count);
        process_simple_hash_style("app.name=MyApp", &mut count);
        process_simple_hash_style("app.version=1.0", &mut count);
        assert_eq!(count, 2);
    }

    // ==================== GraphQL テスト ====================

    #[test]
    fn test_graphql_hash_comment() {
        let mut count = 0;
        process_simple_hash_style("# GraphQL schema", &mut count);
        process_simple_hash_style("type Query {", &mut count);
        process_simple_hash_style("  users: [User]", &mut count);
        process_simple_hash_style("}", &mut count);
        assert_eq!(count, 3);
    }
}
