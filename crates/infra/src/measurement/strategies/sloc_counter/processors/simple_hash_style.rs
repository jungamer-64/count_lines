// crates/infra/src/measurement/strategies/sloc_counter/processors/simple_hash_style.rs
//! シンプルな Hash スタイル (#) のコメント処理
//!
//! 対象: Shell, YAML, TOML, Dockerfile, Makefile, Config系など
//!
//! 特徴:
//! - 複雑な文字列処理不要
//! - `"..."` と `'...'` のみ考慮（バッククォートや三重クォートなし）
//! - Docstringや埋め込みドキュメントなし
//! - 高速かつ安全な処理

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

/// 単純な文字列 ("..." / '...') 外で # を検索
/// 
/// Shell/YAML/Config等向けの軽量版。
/// Python の f-string や三重クォートは考慮しない。
pub fn find_hash_outside_simple_string(line: &str) -> Option<usize> {
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

    // ==================== 基本テスト ====================

    #[test]
    fn test_simple_hash_line_comment() {
        let mut count = 0;
        process_simple_hash_style("# comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_simple_hash_line_comment_with_space() {
        let mut count = 0;
        process_simple_hash_style("  # indented comment", &mut count);
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

    #[test]
    fn test_simple_hash_single_quoted_string_with_hash() {
        let mut count = 0;
        process_simple_hash_style("s = 'hello # world'", &mut count);
        assert_eq!(count, 1);
    }

    // ==================== shebang テスト ====================

    #[test]
    fn test_simple_hash_shebang_not_counted() {
        let mut count = 0;
        process_simple_hash_style("#!/bin/bash", &mut count);
        process_simple_hash_style("echo 'hello'", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_simple_hash_shebang_after_code() {
        let mut count = 0;
        process_simple_hash_style("x = 1", &mut count);
        process_simple_hash_style("#!/bin/bash", &mut count);
        // count > 0 の後の #! 行はコメント扱い
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

    // ==================== Shell テスト ====================

    #[test]
    fn test_shell_script() {
        let mut count = 0;
        process_simple_hash_style("#!/bin/bash", &mut count);
        process_simple_hash_style("# Shell script", &mut count);
        process_simple_hash_style("echo 'Hello'", &mut count);
        process_simple_hash_style("exit 0", &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_shell_variable_expansion() {
        let mut count = 0;
        process_simple_hash_style("VAR=value", &mut count);
        process_simple_hash_style("echo $VAR", &mut count);
        assert_eq!(count, 2);
    }

    // ==================== YAML テスト ====================

    #[test]
    fn test_yaml_document() {
        let mut count = 0;
        process_simple_hash_style("# YAML config", &mut count);
        process_simple_hash_style("name: MyApp", &mut count);
        process_simple_hash_style("version: 1.0", &mut count);
        process_simple_hash_style("enabled: true  # inline comment", &mut count);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_yaml_string_with_hash() {
        let mut count = 0;
        process_simple_hash_style(r#"description: "Contains # symbol""#, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== TOML テスト ====================

    #[test]
    fn test_toml_document() {
        let mut count = 0;
        process_simple_hash_style("# TOML config", &mut count);
        process_simple_hash_style("[package]", &mut count);
        process_simple_hash_style(r#"name = "myapp""#, &mut count);
        process_simple_hash_style(r#"version = "0.1.0"  # version"#, &mut count);
        assert_eq!(count, 3);
    }

    // ==================== Dockerfile テスト ====================

    #[test]
    fn test_dockerfile() {
        let mut count = 0;
        process_simple_hash_style("# Dockerfile", &mut count);
        process_simple_hash_style("FROM ubuntu:20.04", &mut count);
        process_simple_hash_style("RUN apt-get update", &mut count);
        process_simple_hash_style("COPY . /app  # copy files", &mut count);
        assert_eq!(count, 3);
    }

    // ==================== Makefile テスト ====================

    #[test]
    fn test_makefile() {
        let mut count = 0;
        process_simple_hash_style("# Makefile", &mut count);
        process_simple_hash_style("CC = gcc", &mut count);
        process_simple_hash_style("all: main.o", &mut count);
        process_simple_hash_style("\tgcc -o main main.o  # link", &mut count);
        assert_eq!(count, 3);
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

    // ==================== find_hash_outside_simple_string テスト ====================

    #[test]
    fn test_find_hash_no_string() {
        assert_eq!(find_hash_outside_simple_string("x = 1 # comment"), Some(6));
    }

    #[test]
    fn test_find_hash_in_double_string() {
        assert_eq!(find_hash_outside_simple_string(r#""hello # world""#), None);
    }

    #[test]
    fn test_find_hash_in_single_string() {
        assert_eq!(find_hash_outside_simple_string("'hello # world'"), None);
    }

    #[test]
    fn test_find_hash_after_string() {
        let result = find_hash_outside_simple_string(r#""test" # comment"#);
        assert_eq!(result, Some(7));
    }

    #[test]
    fn test_find_hash_escaped_quote() {
        // "hello \" # world" - \" はエスケープなので文字列は終わらない
        assert_eq!(find_hash_outside_simple_string(r#""hello \" # world""#), None);
    }

    #[test]
    fn test_find_hash_no_hash() {
        assert_eq!(find_hash_outside_simple_string("x = 1"), None);
    }

    #[test]
    fn test_find_hash_at_start() {
        assert_eq!(find_hash_outside_simple_string("# comment"), Some(0));
    }

    #[test]
    fn test_find_hash_multiple_strings() {
        let result = find_hash_outside_simple_string(r#""a" + "b" # comment"#);
        assert_eq!(result, Some(10));
    }
}
