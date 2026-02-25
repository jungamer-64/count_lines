// crates/core/src/language/string_utils/tests.rs
use crate::language::StringSkipOptions;
use crate::language::string_utils::PatternMatch;
use crate::language::string_utils::find_any_outside_string;
// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 正規表現リテラルのテスト
    // =========================================================================

    mod regex_literal_tests {
        use crate::language::string_utils::try_skip_regex_literal;

        #[test]
        fn test_simple_regex_literal() {
            // 単純な正規表現リテラル
            let bytes = b"/abc/";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_with_flags() {
            // フラグ付き正規表現
            let bytes = b"/abc/gi";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(7));
        }

        #[test]
        fn test_regex_with_escaped_slash() {
            // エスケープされたスラッシュ
            let bytes = b"/a\\/b/";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(6));
        }

        #[test]
        fn test_regex_with_character_class() {
            // 文字クラス内のスラッシュ
            let bytes = b"/[/]/";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_containing_comment_like_pattern() {
            // 正規表現内の // パターン
            let bytes = b"/https:\\/\\//";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(12));
        }

        #[test]
        fn test_division_after_number() {
            // 数値の後の除算
            let bytes = b"10/2";
            let result = try_skip_regex_literal(bytes, 2);
            assert_eq!(result, None);
        }

        #[test]
        fn test_division_after_identifier() {
            // 識別子の後の除算
            let bytes = b"x/2";
            let result = try_skip_regex_literal(bytes, 1);
            assert_eq!(result, None);
        }

        #[test]
        fn test_division_after_closing_paren() {
            // 閉じ括弧の後の除算
            let bytes = b"(x+y)/2";
            let result = try_skip_regex_literal(bytes, 5);
            assert_eq!(result, None);
        }

        #[test]
        fn test_regex_after_equals() {
            // = の後の正規表現
            let bytes = b"x = /abc/";
            let result = try_skip_regex_literal(bytes, 4);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_after_return() {
            // return の後の正規表現
            let bytes = b"return /abc/";
            let result = try_skip_regex_literal(bytes, 7);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_after_open_paren() {
            // ( の後の正規表現
            let bytes = b"if (/abc/.test(s))";
            let result = try_skip_regex_literal(bytes, 4);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_empty_regex_treated_as_division() {
            // 空の正規表現は除算として扱う
            let bytes = b"//";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, None);
        }

        #[test]
        fn test_regex_at_line_start() {
            // 行頭の正規表現
            let bytes = b"/abc/g.test(x)";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(6));
        }
    }

    // =========================================================================
    // find_outside_string_with_options のテスト (JavaScript/正規表現)
    // =========================================================================

    mod find_outside_string_js_tests {
        use crate::language::StringSkipOptions;
        use crate::language::string_utils::find_outside_string_with_options;

        #[test]
        fn test_js_regex_not_mistaken_for_comment() {
            // 正規表現内の // が行コメントと誤認されないこと
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var re = /https:\\/\\//;", "//", options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_js_comment_after_regex() {
            // 正規表現の後の行コメント
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var re = /abc/g; // comment", "//", options);
            assert_eq!(result, Some(17));
        }

        #[test]
        fn test_js_block_comment_in_regex() {
            // 正規表現内の /* */ がブロックコメントと誤認されないこと
            let options = StringSkipOptions::javascript();
            let result = find_outside_string_with_options("var re = /a*b/g;", "/*", options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_js_division_not_regex() {
            // 除算演算子の後のコメント
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var x = a/b; // division", "//", options);
            assert_eq!(result, Some(13));
        }

        #[test]
        fn test_js_template_string_with_regex() {
            // テンプレート文字列内の正規表現パターン
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var s = `pattern: /abc/`;", "//", options);
            assert_eq!(result, None);
        }
    }

    // =========================================================================
    // find_any_outside_string (multi-pattern search) tests
    // =========================================================================

    mod find_any_outside_string_tests {
        use super::*;

        #[test]
        fn test_find_first_pattern() {
            let options = StringSkipOptions::default();
            let patterns = ["//", "/*"];
            let result = find_any_outside_string("int x = 1; // comment", &patterns, options);
            assert_eq!(
                result,
                Some(PatternMatch {
                    position: 11,
                    pattern_index: 0
                })
            );
        }

        #[test]
        fn test_find_block_comment_first() {
            let options = StringSkipOptions::default();
            let patterns = ["//", "/*"];
            let result = find_any_outside_string("int x = 1; /* block */", &patterns, options);
            assert_eq!(
                result,
                Some(PatternMatch {
                    position: 11,
                    pattern_index: 1
                })
            );
        }

        #[test]
        fn test_pattern_priority() {
            // When both patterns match at same position, first one wins
            let options = StringSkipOptions::default();
            let patterns = ["//", "/"];
            let result = find_any_outside_string("x // y", &patterns, options);
            assert_eq!(
                result,
                Some(PatternMatch {
                    position: 2,
                    pattern_index: 0
                })
            );
        }

        #[test]
        fn test_no_match() {
            let options = StringSkipOptions::default();
            let patterns = ["//", "/*"];
            let result = find_any_outside_string("int x = 1;", &patterns, options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_empty_patterns() {
            let options = StringSkipOptions::default();
            let patterns: [&str; 0] = [];
            let result = find_any_outside_string("int x = 1;", &patterns, options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_empty_line() {
            let options = StringSkipOptions::default();
            let patterns = ["//", "/*"];
            let result = find_any_outside_string("", &patterns, options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_patterns_inside_string() {
            let options = StringSkipOptions::basic();
            let patterns = ["//", "/*"];
            let result =
                find_any_outside_string(r#"let s = "// /* not comment";"#, &patterns, options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_pattern_after_string() {
            let options = StringSkipOptions::basic();
            let patterns = ["//", "/*"];
            let result =
                find_any_outside_string(r#"let s = "text"; // comment"#, &patterns, options);
            assert_eq!(
                result,
                Some(PatternMatch {
                    position: 16,
                    pattern_index: 0
                })
            );
        }

        #[test]
        fn test_multiple_patterns_various_lengths() {
            let options = StringSkipOptions::default();
            let patterns = ["/*", "*/", "//", "#"];
            let result = find_any_outside_string("x # y", &patterns, options);
            assert_eq!(
                result,
                Some(PatternMatch {
                    position: 2,
                    pattern_index: 3
                })
            );
        }

        #[test]
        fn test_rust_options() {
            let options = StringSkipOptions::rust();
            let patterns = ["//", "/*"];
            // Raw string should be skipped
            let result = find_any_outside_string(
                r#"let s = r"// not a comment"; // real"#,
                &patterns,
                options,
            );
            assert_eq!(
                result,
                Some(PatternMatch {
                    position: 29,
                    pattern_index: 0
                })
            );
        }
    }
}
