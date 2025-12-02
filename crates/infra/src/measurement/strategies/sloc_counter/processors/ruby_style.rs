// crates/infra/src/measurement/strategies/sloc_counter/processors/ruby_style.rs
//! Ruby言語のコメント処理
//!
//! Ruby固有の対応:
//! - `#` 行コメント
//! - 埋め込みドキュメント: `=begin` ～ `=end` (行頭必須)
//! - 文字列: `"..."`, `'...'` のみ考慮

use super::simple_hash_style::find_hash_outside_simple_string;

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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Ruby 行コメントテスト ====================

    #[test]
    fn test_ruby_line_comment() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style("# comment", &mut in_doc, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_ruby_line_comment_with_space() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style("  # indented comment", &mut in_doc, &mut count);
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
    fn test_ruby_code_with_inline_comment() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style("x = 1  # inline comment", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== Ruby 埋め込みドキュメントテスト ====================

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

        // =begin がインデントされていると埋め込みドキュメント開始として認識されない
        process_ruby_style("  =begin", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_ruby_style("x = 1", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ruby_embedded_doc_in_comment() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("x = 1  # =begin is not at line start", &mut in_doc, &mut count);
        process_ruby_style("y = 2", &mut in_doc, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ruby_embedded_doc_empty() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("=begin", &mut in_doc, &mut count);
        assert!(in_doc);
        process_ruby_style("=end", &mut in_doc, &mut count);
        assert!(!in_doc);
        process_ruby_style("x = 1", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== Ruby shebang テスト ====================

    #[test]
    fn test_ruby_shebang_not_counted() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("#!/usr/bin/env ruby", &mut in_doc, &mut count);
        process_ruby_style("puts 'Hello'", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ruby_shebang_after_code() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("x = 1", &mut in_doc, &mut count);
        process_ruby_style("#!/usr/bin/env ruby", &mut in_doc, &mut count);
        // count > 0 の状態では shebang もコード行として扱われる可能性あり
        // ただし、#! で始まるのでコメントとして扱われる
        assert_eq!(count, 1);
    }

    // ==================== Ruby 文字列内 # テスト ====================

    #[test]
    fn test_ruby_string_with_hash() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style(r#"s = "hello # world""#, &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ruby_single_quoted_string_with_hash() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style("s = 'hello # world'", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ruby_string_with_hash_and_real_comment() {
        let mut in_doc = false;
        let mut count = 0;
        process_ruby_style(r#"s = "test" # real comment"#, &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== Ruby クラス・メソッド定義テスト ====================

    #[test]
    fn test_ruby_class_definition() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("# Ruby class", &mut in_doc, &mut count);
        process_ruby_style("class Foo", &mut in_doc, &mut count);
        process_ruby_style("  def bar", &mut in_doc, &mut count);
        process_ruby_style("    @x = 1 # instance var", &mut in_doc, &mut count);
        process_ruby_style("  end", &mut in_doc, &mut count);
        process_ruby_style("end", &mut in_doc, &mut count);
        assert_eq!(count, 5);
    }

    #[test]
    fn test_ruby_attr_accessor() {
        let mut in_doc = false;
        let mut count = 0;

        process_ruby_style("attr_accessor :name # accessor", &mut in_doc, &mut count);
        assert_eq!(count, 1);
    }
}
