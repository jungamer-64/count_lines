// crates/infra/src/measurement/strategies/sloc_counter.rs
//! SLOC (Source Lines of Code) カウンター
//!
//! 言語ごとのコメント構文を認識し、純粋なコード行のみをカウントします。

mod comment_style;
mod processors;
mod string_utils;

pub use comment_style::CommentStyle;

use processors::{
    process_c_style, process_cpp_style, process_erlang_style, process_fortran_style,
    process_hash_style, process_haskell_style, process_html_style, process_julia_style,
    process_lisp_style, process_lua_style, process_matlab_style, process_nesting_c_style,
    process_ocaml_style, process_php_style, process_powershell_style, process_sql_style,
    process_swift_style,
};

/// SLOCカウンターの状態
pub struct SlocCounter {
    style: CommentStyle,
    in_block_comment: bool,
    /// Rust/Swift/Kotlin/Haskellのネストされたブロックコメント用の深さカウンター
    block_comment_depth: usize,
    /// ブロックコメントのネストをサポートするか (Rust/Swift/Kotlin)
    supports_nesting: bool,
    /// Python Docstringの開始クォート (Some(b'"') or Some(b'\''))
    docstring_quote: Option<u8>,
    /// Ruby/Perl の埋め込みドキュメント内か (=begin/=end, =pod/=cut)
    in_embedded_doc: bool,
    /// C++ Raw Stringリテラルをサポートするか
    supports_cpp_raw_string: bool,
    /// Swift の拡張デリミタ文字列をサポートするか
    is_swift: bool,
    /// Lua ブロックコメントのレベル (等号の数)
    lua_block_level: usize,
    count: usize,
}

impl SlocCounter {
    /// 新しいカウンターを作成
    pub fn new(extension: &str) -> Self {
        let style = CommentStyle::from_extension(extension);
        let ext_lower = extension.to_lowercase();

        // ネストコメントをサポートする言語
        let supports_nesting = matches!(
            ext_lower.as_str(),
            "rs" | "swift" | "kt" | "kts" | "scala" | "sc" | "d"
        );

        // C++ Raw Stringをサポートする言語
        let supports_cpp_raw_string = matches!(
            ext_lower.as_str(),
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++"
        );

        // Swift の拡張デリミタ文字列をサポート
        let is_swift = ext_lower == "swift";

        Self {
            style,
            in_block_comment: false,
            block_comment_depth: 0,
            supports_nesting,
            docstring_quote: None,
            in_embedded_doc: false,
            supports_cpp_raw_string,
            is_swift,
            lua_block_level: 0,
            count: 0,
        }
    }

    /// 行を処理してSLOCかどうかを判定
    pub fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();

        // 空行はスキップ
        if trimmed.is_empty() {
            return;
        }

        match self.style {
            CommentStyle::CStyle => {
                // Swift は拡張デリミタ文字列対応の専用処理を使用
                if self.is_swift {
                    process_swift_style(
                        trimmed,
                        &mut self.block_comment_depth,
                        &mut self.in_block_comment,
                        &mut self.count,
                    );
                } else if self.supports_nesting {
                    // Rust/Kotlin/Scala/D などのネストコメント対応
                    process_nesting_c_style(
                        trimmed,
                        &mut self.block_comment_depth,
                        &mut self.in_block_comment,
                        &mut self.count,
                    );
                } else if self.supports_cpp_raw_string {
                    process_cpp_style(trimmed, &mut self.in_block_comment, &mut self.count);
                } else {
                    process_c_style(trimmed, &mut self.in_block_comment, &mut self.count);
                }
            }
            CommentStyle::Hash => process_hash_style(
                line, // 埋め込みドキュメントは行頭判定が必要なため trim 前の line を渡す
                &mut self.docstring_quote,
                &mut self.in_embedded_doc,
                &mut self.in_block_comment,
                &mut self.count,
            ),
            CommentStyle::Php => {
                process_php_style(trimmed, &mut self.in_block_comment, &mut self.count)
            }
            CommentStyle::PowerShell => {
                process_powershell_style(trimmed, &mut self.in_block_comment, &mut self.count)
            }
            CommentStyle::Lua => {
                process_lua_style(
                    trimmed,
                    &mut self.in_block_comment,
                    &mut self.lua_block_level,
                    &mut self.count,
                )
            }
            CommentStyle::Html => {
                process_html_style(trimmed, &mut self.in_block_comment, &mut self.count)
            }
            CommentStyle::Sql => {
                process_sql_style(trimmed, &mut self.in_block_comment, &mut self.count)
            }
            CommentStyle::Haskell => {
                process_haskell_style(
                    trimmed,
                    &mut self.in_block_comment,
                    &mut self.block_comment_depth,
                    &mut self.count,
                )
            }
            CommentStyle::Julia => {
                process_julia_style(
                    trimmed,
                    &mut self.in_block_comment,
                    &mut self.block_comment_depth,
                    &mut self.count,
                )
            }
            CommentStyle::OCaml => {
                process_ocaml_style(
                    trimmed,
                    &mut self.in_block_comment,
                    &mut self.block_comment_depth,
                    &mut self.count,
                )
            }
            CommentStyle::Lisp => process_lisp_style(trimmed, &mut self.count),
            CommentStyle::Erlang => process_erlang_style(trimmed, &mut self.count),
            CommentStyle::Fortran => process_fortran_style(trimmed, &mut self.count),
            CommentStyle::Matlab => {
                process_matlab_style(trimmed, &mut self.in_block_comment, &mut self.count)
            }
            CommentStyle::None => {
                // コメント構文なしの場合、非空行は全てSLOC
                self.count += 1;
            }
        }
    }

    /// 現在のSLOCカウントを取得
    pub fn count(&self) -> usize {
        self.count
    }

    /// ブロックコメント内かどうか（テスト用）
    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_style_single_line_comment() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("// this is a comment");
        counter.process_line("let x = 1; // inline comment");
        counter.process_line("let y = 2;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_rust_doc_comments() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("//! Module doc comment");
        counter.process_line("//!");
        counter.process_line("//! Another line");
        counter.process_line("");
        counter.process_line("/// Function doc comment");
        counter.process_line("/// More docs");
        counter.process_line("pub fn foo() {}");
        // Doc comments should not be counted, only the actual code line
        assert_eq!(
            counter.count(),
            1,
            "Only 'pub fn foo() {{}}' should be counted as SLOC"
        );
    }

    #[test]
    fn test_rust_realistic_file() {
        let mut counter = SlocCounter::new("rs");
        let lines = vec![
            "//! Security Policy Engine",
            "//!",
            "//! This module implements security policy.",
            "",
            "use core::fmt;",
            "use alloc::vec::Vec;",
            "",
            "/// Policy action to take",
            "#[derive(Debug, Clone)]",
            "pub enum PolicyAction {",
            "    /// Allow the operation",
            "    Allow,",
            "    /// Deny the operation",
            "    Deny,",
            "}",
            "",
            "impl PolicyAction {",
            "    /// Check if action allows",
            "    pub fn is_allow(&self) -> bool {",
            "        matches!(self, PolicyAction::Allow)",
            "    }",
            "}",
        ];

        for line in &lines {
            counter.process_line(line);
        }

        // Expected SLOC: use(2) + #[derive](1) + enum declaration(1) + Allow,(1) + Deny,(1) + }(1)
        //              + impl(1) + pub fn(1) + matches!(1) + }(1) + }(1) = 12
        // NOT including: //!, //! comments, /// doc comments, empty lines
        assert!(
            counter.count() > 10,
            "Expected more than 10 SLOC, got {}",
            counter.count()
        );
    }

    #[test]
    fn test_attribute_and_code_mixed() {
        // Test that attributes like #[derive(...)] are counted as SLOC
        let mut counter = SlocCounter::new("rs");
        counter.process_line("#[derive(Debug, Clone)]");
        counter.process_line("pub struct Foo;");
        assert_eq!(
            counter.count(),
            2,
            "Both attribute and struct should be SLOC"
        );
    }

    #[test]
    fn test_comment_in_string_literal() {
        // Test that /* inside string literals is not treated as block comment
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"if pattern.ends_with("/*") {"#);
        counter.process_line("    // do something");
        counter.process_line("}");
        // First line has code, second is comment, third has code
        assert_eq!(
            counter.count(),
            2,
            "String literal with /* should not trigger block comment"
        );
        assert!(
            !counter.is_in_block_comment(),
            "Should not be in block comment mode"
        );
    }

    #[test]
    fn test_comment_in_string_literal_double_star() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"} else if pattern.ends_with("/**") {"#);
        counter.process_line("    let x = 1;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_rust_raw_string_literal() {
        // r"..." 形式のraw文字列
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"let s = r"/* not a comment */";"#);
        counter.process_line("let x = 1;");
        assert_eq!(counter.count(), 2);
        assert!(
            !counter.is_in_block_comment(),
            "r\"...\" should not trigger block comment"
        );
    }

    #[test]
    fn test_rust_raw_string_with_hashes() {
        // r#"..."# 形式のraw文字列
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r###"let regex = r#"<div class="foo">"#;"###);
        counter.process_line("let y = 2;");
        assert_eq!(counter.count(), 2);
        assert!(
            !counter.is_in_block_comment(),
            "r#\"...\"# should not trigger block comment"
        );
    }

    #[test]
    fn test_rust_raw_string_with_multiple_hashes() {
        // r##"..."## 形式のraw文字列
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r####"let s = r##"contains "# but not end"##;"####);
        counter.process_line("let z = 3;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_rust_raw_string_with_comment_markers() {
        // raw文字列内に // や /* */ が含まれるケース
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r##"let pattern = r#"// not comment /* also not */ //"#;"##);
        counter.process_line("real_code();");
        counter.process_line("// actual comment");
        counter.process_line("more_code();");
        assert_eq!(counter.count(), 3); // raw文字列行 + real_code + more_code
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_block_comment_state_issue() {
        // Test the actual content from the user's file
        let mut counter = SlocCounter::new("rs");

        // These lines should NOT trigger block comment mode
        let test_lines = vec![
            "//! Security Policy Engine for ExoRust",
            "//!",
            "//! This module implements a flexible rule-based security policy",
            "//! system for controlling access and operations.",
            "",
            "use core::fmt;",
            "use alloc::vec::Vec;",
            "use alloc::string::String;",
            "use alloc::collections::BTreeMap;",
            "use spin::RwLock;",
            "",
            "extern crate alloc;",
            "",
            "/// Policy action to take",
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
            "pub enum PolicyAction {",
            "    /// Allow the operation",
            "    Allow,",
        ];

        for (i, line) in test_lines.iter().enumerate() {
            let before = counter.count();
            counter.process_line(line);
            let after = counter.count();
            eprintln!(
                "Line {}: '{}' -> count {} -> {} (in_block={})",
                i,
                line,
                before,
                after,
                counter.is_in_block_comment()
            );
        }

        // Should have: 5 use statements + extern crate + #[derive] + pub enum + Allow, = 9
        assert!(
            counter.count() >= 9,
            "Expected at least 9 SLOC, got {}",
            counter.count()
        );
        assert!(
            !counter.is_in_block_comment(),
            "Should not be in block comment mode"
        );
    }

    #[test]
    fn test_c_style_block_comment() {
        let mut counter = SlocCounter::new("c");
        counter.process_line("/* block comment */");
        counter.process_line("int x = 1;");
        counter.process_line("/* multi");
        counter.process_line("   line");
        counter.process_line("   comment */");
        counter.process_line("int y = 2;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_python_hash_comment() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("#!/usr/bin/env python");
        counter.process_line("# comment");
        counter.process_line("x = 1  # inline");
        counter.process_line("y = 2");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_empty_lines_ignored() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("");
        counter.process_line("   ");
        counter.process_line("\t");
        counter.process_line("let x = 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_unknown_extension() {
        let mut counter = SlocCounter::new("xyz");
        counter.process_line("any content");
        counter.process_line("// this is not treated as comment");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_html_comment() {
        let mut counter = SlocCounter::new("html");
        counter.process_line("<!-- comment -->");
        counter.process_line("<div>content</div>");
        counter.process_line("<!-- multi");
        counter.process_line("line -->");
        counter.process_line("<p>text</p>");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_lua_comment() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("-- comment");
        counter.process_line("local x = 1");
        counter.process_line("--[[ block");
        counter.process_line("comment ]]");
        counter.process_line("local y = 2");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_identifier_ending_with_r_not_raw_string() {
        // bar"/*" のような識別子+文字列がraw文字列として誤検出されないこと
        let mut counter = SlocCounter::new("rs");
        // bar という変数に文字列を代入（r で終わるが raw 文字列ではない）
        counter.process_line(r#"let bar = "/*";"#);
        counter.process_line("let x = 1;");
        // "/*" は通常文字列なので、ブロックコメント開始として誤検出されない
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_byte_string_literal() {
        // b"..." 形式のバイト文字列
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"let bytes = b"/* not a comment */";"#);
        counter.process_line("let x = 1;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_byte_raw_string_literal() {
        // br"..." や br#"..."# 形式のバイトraw文字列
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r##"let bytes = br#"/* not a comment */"#;"##);
        counter.process_line("let y = 2;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_identifier_with_r_suffix_complex() {
        // 複雑なケース: 識別子が r で終わり、その後に文字列がある
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"println!(buffer"/*test*/");"#); // 架空の構文
                                                                // これは raw 文字列ではないので通常文字列として処理される
        assert_eq!(counter.count(), 1);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_lifetime_annotation_not_char_literal() {
        // ライフタイム注釈が文字リテラルとして誤認されないこと
        let mut counter = SlocCounter::new("rs");
        counter.process_line("fn foo<'a, 'b>(x: &'a str, y: &'b str) {");
        counter.process_line("    x.len() // コメント");
        counter.process_line("}");
        // 全て SLOC（コメント付きの行もコードがあるので）
        assert_eq!(counter.count(), 3);
    }

    #[test]
    fn test_lifetime_static() {
        // 'static ライフタイム
        let mut counter = SlocCounter::new("rs");
        counter.process_line("static S: &'static str = \"hello\";");
        counter.process_line("let x = 1;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_char_literal_vs_lifetime() {
        // 文字リテラルとライフタイムの混在
        let mut counter = SlocCounter::new("rs");
        counter.process_line("fn foo<'a>(c: char) -> &'a str {");
        counter.process_line("    let x = 'c';"); // 文字リテラル
        counter.process_line("    let y = '\\n';"); // エスケープ文字リテラル
        counter.process_line("}");
        assert_eq!(counter.count(), 4);
    }

    #[test]
    fn test_nested_block_comment() {
        // Rustのネストされたブロックコメント
        let mut counter = SlocCounter::new("rs");
        counter.process_line("/* outer");
        counter.process_line("  /* inner */");
        counter.process_line("  still in comment");
        counter.process_line("*/");
        counter.process_line("let x = 1;"); // ここからコード
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_nested_block_comment_single_line() {
        // 1行にネストされたコメント
        let mut counter = SlocCounter::new("rs");
        counter.process_line("/* /* nested */ still comment */ code();");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_nested_block_comment_deep() {
        // 深いネスト
        let mut counter = SlocCounter::new("rs");
        counter.process_line("/* level 1");
        counter.process_line("/* level 2");
        counter.process_line("/* level 3 */");
        counter.process_line("back to level 2 */");
        counter.process_line("back to level 1 */");
        counter.process_line("code();");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_code_before_nested_comment() {
        // コードの後にネストコメント開始
        let mut counter = SlocCounter::new("rs");
        counter.process_line("let x = 1; /* comment");
        counter.process_line("  /* nested */");
        counter.process_line("*/");
        counter.process_line("let y = 2;");
        assert_eq!(counter.count(), 2); // x = 1 と y = 2 の2行
    }

    // ==================== C系言語ネスト非対応テスト ====================

    #[test]
    fn test_c_no_nested_comments() {
        // C言語はネストコメント非対応
        let mut counter = SlocCounter::new("c");
        counter.process_line("/* outer /* inner */ code_here(); */");
        // C言語では最初の */ でコメント終了、code_here(); がコード扱い
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_java_no_nested_comments() {
        // Javaもネストコメント非対応
        let mut counter = SlocCounter::new("java");
        counter.process_line("/* comment */");
        counter.process_line("int x = 1;");
        assert_eq!(counter.count(), 1);
    }

    // ==================== Python Docstring テスト ====================

    #[test]
    fn test_python_docstring_multiline() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("def foo():");
        counter.process_line("    \"\"\"");
        counter.process_line("    This is a docstring.");
        counter.process_line("    Multiple lines.");
        counter.process_line("    \"\"\"");
        counter.process_line("    return 1");
        // def foo(): と return 1 のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_python_docstring_single_line() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("def bar():");
        counter.process_line("    \"\"\"Single line docstring.\"\"\"");
        counter.process_line("    pass");
        // def bar(): と pass のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_python_docstring_single_quote() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("'''");
        counter.process_line("Triple single quote docstring");
        counter.process_line("'''");
        counter.process_line("x = 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_python_string_with_hash() {
        // 文字列内の # はコメントではない
        let mut counter = SlocCounter::new("py");
        counter.process_line("s = \"hello # world\"");
        counter.process_line("t = 'foo # bar'");
        assert_eq!(counter.count(), 2);
    }

    // ==================== C++ Raw String Literal テスト ====================

    #[test]
    fn test_cpp_raw_string_basic() {
        let mut counter = SlocCounter::new("cpp");
        counter.process_line("const char* s = R\"(/* not a comment */)\";");
        counter.process_line("int x = 1;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_cpp_raw_string_with_delimiter() {
        let mut counter = SlocCounter::new("cpp");
        counter.process_line("const char* s = R\"foo(/* not a comment */)foo\";");
        counter.process_line("int y = 2;");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_cpp_raw_string_with_line_comment() {
        let mut counter = SlocCounter::new("cpp");
        counter.process_line("const char* s = R\"(// not a comment)\";");
        counter.process_line("int z = 3;");
        assert_eq!(counter.count(), 2);
    }

    // ==================== Swift/Kotlin ネストコメントテスト ====================

    #[test]
    fn test_swift_nested_comments() {
        let mut counter = SlocCounter::new("swift");
        counter.process_line("/* outer");
        counter.process_line("  /* nested */");
        counter.process_line("*/");
        counter.process_line("let x = 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_kotlin_nested_comments() {
        let mut counter = SlocCounter::new("kt");
        counter.process_line("/* outer /* inner */ still comment */");
        counter.process_line("val x = 1");
        // Kotlinはネスト対応なので、全てのコメントが閉じてからコード
        assert_eq!(counter.count(), 1);
    }

    // ==================== Ruby 埋め込みドキュメント テスト ====================

    #[test]
    fn test_ruby_embedded_document() {
        let mut counter = SlocCounter::new("rb");
        counter.process_line("x = 1");
        counter.process_line("=begin");
        counter.process_line("This is embedded documentation.");
        counter.process_line("It can span multiple lines.");
        counter.process_line("=end");
        counter.process_line("y = 2");
        // x = 1 と y = 2 のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_ruby_embedded_document_with_comments() {
        let mut counter = SlocCounter::new("rb");
        counter.process_line("# regular comment");
        counter.process_line("def foo");
        counter.process_line("=begin");
        counter.process_line("  embedded doc");
        counter.process_line("=end");
        counter.process_line("  puts 'hello'");
        counter.process_line("end");
        // def foo, puts 'hello', end のみがSLOC
        assert_eq!(counter.count(), 3);
    }

    #[test]
    fn test_ruby_embedded_doc_must_start_at_line_beginning() {
        // =begin は行頭から始まる必要がある
        let mut counter = SlocCounter::new("rb");
        counter.process_line("x = 1  # =begin is not at line start");
        counter.process_line("y = 2");
        // 両方ともコード行
        assert_eq!(counter.count(), 2);
    }

    // ==================== Perl POD テスト ====================

    #[test]
    fn test_perl_pod_basic() {
        let mut counter = SlocCounter::new("pl");
        counter.process_line("use strict;");
        counter.process_line("=pod");
        counter.process_line("This is POD documentation.");
        counter.process_line("=cut");
        counter.process_line("print \"Hello\";");
        // use strict と print のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_perl_pod_with_head() {
        let mut counter = SlocCounter::new("pl");
        counter.process_line("my $x = 1;");
        counter.process_line("=head1 NAME");
        counter.process_line("");
        counter.process_line("MyModule - A sample module");
        counter.process_line("");
        counter.process_line("=head2 DESCRIPTION");
        counter.process_line("");
        counter.process_line("This module does something.");
        counter.process_line("");
        counter.process_line("=cut");
        counter.process_line("my $y = 2;");
        // $x = 1 と $y = 2 のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_perl_pod_over_item() {
        let mut counter = SlocCounter::new("pm");
        counter.process_line("sub foo { 1 }");
        counter.process_line("=over 4");
        counter.process_line("=item * First item");
        counter.process_line("=item * Second item");
        counter.process_line("=back");
        counter.process_line("=cut");
        counter.process_line("sub bar { 2 }");
        // sub foo と sub bar のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_perl_shebang_not_counted() {
        let mut counter = SlocCounter::new("pl");
        counter.process_line("#!/usr/bin/perl");
        counter.process_line("use warnings;");
        counter.process_line("print 1;");
        // shebang は除外、use と print がSLOC
        assert_eq!(counter.count(), 2);
    }

    // ==================== PowerShell テスト ====================

    #[test]
    fn test_powershell_line_comment() {
        let mut counter = SlocCounter::new("ps1");
        counter.process_line("# This is a comment");
        counter.process_line("$x = 1");
        counter.process_line("$y = 2  # inline comment");
        // $x と $y の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_powershell_block_comment_single_line() {
        let mut counter = SlocCounter::new("ps1");
        counter.process_line("<# block comment #>");
        counter.process_line("$x = 1");
        // ブロックコメントは除外、$x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_powershell_block_comment_multiline() {
        let mut counter = SlocCounter::new("ps1");
        counter.process_line("$x = 1");
        counter.process_line("<#");
        counter.process_line("  This is a multi-line");
        counter.process_line("  block comment");
        counter.process_line("#>");
        counter.process_line("$y = 2");
        // $x と $y の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_powershell_code_before_block_comment() {
        let mut counter = SlocCounter::new("ps1");
        counter.process_line("$x = 1 <# start comment");
        counter.process_line("still in comment");
        counter.process_line("#> $y = 2");
        // $x = 1 と $y = 2 の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_powershell_block_comment_in_string() {
        let mut counter = SlocCounter::new("ps1");
        // 文字列内の <# はコメント開始ではない
        counter.process_line(r#"$s = "<# not a comment #>""#);
        counter.process_line("$x = 1");
        // 両方ともSLOC
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_powershell_module() {
        let mut counter = SlocCounter::new("psm1");
        counter.process_line("<#");
        counter.process_line(".SYNOPSIS");
        counter.process_line("    A sample function");
        counter.process_line("#>");
        counter.process_line("function Get-Sample {");
        counter.process_line("    param($Name)");
        counter.process_line("    Write-Output $Name");
        counter.process_line("}");
        // function, param, Write-Output, } の4行がSLOC
        assert_eq!(counter.count(), 4);
    }

    // ==================== Swift 拡張デリミタ文字列テスト ====================

    #[test]
    fn test_swift_extended_delimiter_string() {
        let mut counter = SlocCounter::new("swift");
        // #"..."# 形式の拡張デリミタ文字列
        counter.process_line(r##"let s = #"/* not a comment */"#"##);
        counter.process_line("let x = 1");
        // 両方ともSLOC
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_swift_extended_delimiter_double_hash() {
        let mut counter = SlocCounter::new("swift");
        // ##"..."## 形式
        counter.process_line(r###"let s = ##"contains "# but not end"##"###);
        counter.process_line("let y = 2");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_swift_multiline_string() {
        let mut counter = SlocCounter::new("swift");
        // """...""" 形式の多重引用符文字列（1行で閉じる場合）
        counter.process_line(r#"let s = """/* not a comment */""""#);
        counter.process_line("let z = 3");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_swift_extended_delimiter_with_comment_markers() {
        let mut counter = SlocCounter::new("swift");
        // 拡張デリミタ文字列内にコメントマーカーがある
        counter.process_line(r##"let pattern = #"// not comment /* also not */"#"##);
        counter.process_line("realCode()");
        counter.process_line("// actual comment");
        counter.process_line("moreCode()");
        // pattern, realCode, moreCode の3行がSLOC
        assert_eq!(counter.count(), 3);
    }

    #[test]
    fn test_swift_normal_string_with_comment() {
        let mut counter = SlocCounter::new("swift");
        // 通常の文字列
        counter.process_line(r#"let s = "/* not a comment */""#);
        counter.process_line("let x = 1");
        assert_eq!(counter.count(), 2);
        assert!(!counter.is_in_block_comment());
    }

    #[test]
    fn test_swift_nested_comments_with_strings() {
        // Swift はネストコメントをサポート
        let mut counter = SlocCounter::new("swift");
        counter.process_line("/* outer");
        counter.process_line("  /* nested */");
        counter.process_line("  still in comment");
        counter.process_line("*/");
        counter.process_line("let x = 1");
        // let x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_swift_hash_not_comment() {
        // Swift では # はコメント開始ではない（拡張デリミタの一部）
        let mut counter = SlocCounter::new("swift");
        counter.process_line("let hash = #selector(foo)");
        counter.process_line("let x = 1");
        // 両方ともSLOC
        assert_eq!(counter.count(), 2);
    }

    // ==================== PHP テスト ====================

    #[test]
    fn test_php_hash_comment() {
        let mut counter = SlocCounter::new("php");
        counter.process_line("<?php");
        counter.process_line("# This is a hash comment");
        counter.process_line("$x = 1;");
        counter.process_line("$y = 2; # inline comment");
        counter.process_line("// double slash comment");
        counter.process_line("$z = 3; // inline");
        // <?php, $x, $y, $z の4行がSLOC
        assert_eq!(counter.count(), 4);
    }

    #[test]
    fn test_php_block_comment() {
        let mut counter = SlocCounter::new("php");
        counter.process_line("$a = 1;");
        counter.process_line("/* block");
        counter.process_line("   comment */");
        counter.process_line("$b = 2;");
        // $a と $b の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_php_hash_in_string() {
        let mut counter = SlocCounter::new("php");
        // 文字列内の # はコメントではない
        counter.process_line(r#"$s = "Hello # World";"#);
        counter.process_line(r#"$t = 'Hash: #tag';"#);
        assert_eq!(counter.count(), 2);
    }

    // ==================== Python f-string テスト ====================

    #[test]
    fn test_python_fstring_with_hash() {
        let mut counter = SlocCounter::new("py");
        // f-string 内の # はコメントではない
        counter.process_line(r#"s = f"Hash: #{value}""#);
        counter.process_line("x = 1");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_python_various_prefixes() {
        let mut counter = SlocCounter::new("py");
        counter.process_line(r#"a = f"test # not comment""#);
        counter.process_line(r#"b = F"test # not comment""#);
        counter.process_line(r#"c = r"test # not comment""#);
        counter.process_line(r#"d = u"test # not comment""#);
        counter.process_line(r#"e = b"test # not comment""#);
        counter.process_line(r#"f = fr"test # not comment""#);
        counter.process_line(r#"g = rf"test # not comment""#);
        // 全て7行がSLOC
        assert_eq!(counter.count(), 7);
    }

    #[test]
    fn test_python_fstring_multiline() {
        let mut counter = SlocCounter::new("py");
        counter.process_line(r#"s = f"""Multi"#);
        counter.process_line("line # not comment");
        counter.process_line(r#"string""""#);
        counter.process_line("x = 1");
        // 三重引用符の開始行、終了行、x = 1 の3行がSLOC
        // (中間行は文字列の一部なのでSLOCとしてカウント)
        assert_eq!(counter.count(), 4);
    }

    // ==================== Haskell ネストコメント テスト ====================

    #[test]
    fn test_haskell_nested_comment() {
        let mut counter = SlocCounter::new("hs");
        counter.process_line("{-");
        counter.process_line("  Outer comment");
        counter.process_line("  {- Inner comment -}");
        counter.process_line("  Still outer comment");
        counter.process_line("-}");
        counter.process_line("main = putStrLn \"Hello\"");
        // main の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_haskell_nested_comment_deep() {
        let mut counter = SlocCounter::new("hs");
        counter.process_line("{- level 1");
        counter.process_line("{- level 2");
        counter.process_line("{- level 3 -}");
        counter.process_line("back to level 2 -}");
        counter.process_line("back to level 1 -}");
        counter.process_line("x = 1");
        // x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_haskell_nested_comment_single_line() {
        let mut counter = SlocCounter::new("hs");
        counter.process_line("{- {- nested -} still comment -} x = 1");
        // x = 1 の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_haskell_line_comment() {
        let mut counter = SlocCounter::new("hs");
        counter.process_line("-- comment");
        counter.process_line("x = 1 -- inline");
        // x = 1 は行コメントの前にコードがあるが、現在の実装では starts_with("--") でチェックしている
        // そのため、このケースはコード行としてカウントされる
        assert_eq!(counter.count(), 1);
    }

    // ==================== Lua 任意深さコメント テスト ====================

    #[test]
    fn test_lua_block_comment_basic() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("--[[");
        counter.process_line("  block comment");
        counter.process_line("]]");
        counter.process_line("local x = 1");
        // local x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_lua_block_comment_level_1() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("--[=[");
        counter.process_line("  contains ]] but not end");
        counter.process_line("]=]");
        counter.process_line("local y = 2");
        // local y の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_lua_block_comment_level_3() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("--[===[");
        counter.process_line("  contains ]] and ]=] but not end");
        counter.process_line("]===]");
        counter.process_line("local z = 3");
        // local z の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_lua_block_comment_single_line() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("--[[ single line block ]]");
        counter.process_line("local a = 1");
        // local a の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_lua_line_comment() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("-- line comment");
        counter.process_line("local b = 2");
        // local b の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== Julia テスト ====================

    #[test]
    fn test_julia_line_comment() {
        let mut counter = SlocCounter::new("jl");
        counter.process_line("# comment");
        counter.process_line("x = 1");
        counter.process_line("y = 2 # inline comment");
        // x と y の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_julia_block_comment() {
        let mut counter = SlocCounter::new("jl");
        counter.process_line("#=");
        counter.process_line("  block comment");
        counter.process_line("=#");
        counter.process_line("z = 3");
        // z の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_julia_nested_block_comment() {
        let mut counter = SlocCounter::new("jl");
        counter.process_line("#= outer");
        counter.process_line("#= inner =#");
        counter.process_line("still in outer =#");
        counter.process_line("a = 1");
        // a の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_julia_block_comment_single_line() {
        let mut counter = SlocCounter::new("jl");
        counter.process_line("#= comment =# b = 2");
        // b = 2 の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== OCaml/F#/Pascal テスト ====================

    #[test]
    fn test_ocaml_block_comment() {
        let mut counter = SlocCounter::new("ml");
        counter.process_line("(* comment *)");
        counter.process_line("let x = 1");
        // x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_ocaml_nested_block_comment() {
        let mut counter = SlocCounter::new("ml");
        counter.process_line("(* outer (* inner *) still outer *)");
        counter.process_line("let y = 2");
        // y の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_ocaml_multiline_block_comment() {
        let mut counter = SlocCounter::new("ml");
        counter.process_line("(*");
        counter.process_line("  multiline");
        counter.process_line("*)");
        counter.process_line("let z = 3");
        // z の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_fsharp_line_comment() {
        let mut counter = SlocCounter::new("fs");
        counter.process_line("// F# comment");
        counter.process_line("let a = 1");
        // a の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_pascal_block_comment() {
        let mut counter = SlocCounter::new("pas");
        counter.process_line("(* Pascal comment *)");
        counter.process_line("var x: Integer;");
        // x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== バッククォート文字列テスト ====================

    #[test]
    fn test_go_raw_string_with_comment_marker() {
        let mut counter = SlocCounter::new("go");
        counter.process_line("s := `/* not a comment */`");
        counter.process_line("x := 1");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_js_template_literal_with_comment_marker() {
        let mut counter = SlocCounter::new("js");
        counter.process_line("const s = `// not a comment`;");
        counter.process_line("const x = 1;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_ts_template_literal_with_block_comment() {
        let mut counter = SlocCounter::new("ts");
        counter.process_line("const t = `/* still not a comment */`;");
        counter.process_line("const y = 2;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }
}
