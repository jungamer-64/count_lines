// crates/infra/src/measurement/strategies/sloc_counter.rs
//! SLOC (Source Lines of Code) カウンター
//!
//! 言語ごとのコメント構文を認識し、純粋なコード行のみをカウントします。

mod comment_style;
mod processors;
mod string_utils;

pub use comment_style::CommentStyle;

use processors::{
    process_assembly_style, process_batch_style, process_c_style_with_options,
    process_dlang_style, process_erlang_style, process_fortran_style, process_gas_assembly_style,
    process_haskell_style, process_html_style, process_julia_style, process_lisp_style,
    process_lua_style, process_matlab_style, process_nesting_c_style_with_options,
    process_ocaml_style, process_perl_style, process_php_style, process_powershell_style,
    process_python_style, process_ruby_style, process_simple_hash_style, process_sql_style,
    process_swift_style, process_vhdl_style, process_visual_basic_style,
};
use string_utils::StringSkipOptions;

/// SLOCカウンターの状態
pub struct SlocCounter {
    style: CommentStyle,
    /// 言語に応じた文字列スキップオプション
    string_opts: StringSkipOptions,
    in_block_comment: bool,
    /// Rust/Swift/Kotlin/Haskellのネストされたブロックコメント用の深さカウンター
    block_comment_depth: usize,
    /// ブロックコメントのネストをサポートするか (Rust/Swift/Kotlin)
    supports_nesting: bool,
    /// Python Docstringの開始クォート (Some(b'"') or Some(b'\''))
    docstring_quote: Option<u8>,
    /// Ruby/Perl の埋め込みドキュメント内か (=begin/=end, =pod/=cut)
    in_embedded_doc: bool,
    /// Swift の拡張デリミタ文字列をサポートするか
    is_swift: bool,
    /// Lua ブロックコメントのレベル (等号の数)
    lua_block_level: usize,
    /// D言語の /+ +/ ネストブロックコメント内か
    in_dlang_nesting_block: bool,
    /// D言語の /+ +/ ネストブロックコメントの深さ
    dlang_nesting_depth: usize,
    count: usize,
}

impl SlocCounter {
    /// 新しいカウンターを作成
    pub fn new(extension: &str) -> Self {
        let style = CommentStyle::from_extension(extension);
        let ext_lower = extension.to_lowercase();

        // 言語に応じた文字列スキップオプションを設定
        let string_opts = StringSkipOptions::from_extension(extension);

        // ネストコメントをサポートする言語 (D言語は別処理)
        let supports_nesting = matches!(
            ext_lower.as_str(),
            "rs" | "swift" | "kt" | "kts" | "scala" | "sc"
        );

        // Swift の拡張デリミタ文字列をサポート
        let is_swift = ext_lower == "swift";

        Self {
            style,
            string_opts,
            in_block_comment: false,
            block_comment_depth: 0,
            supports_nesting,
            docstring_quote: None,
            in_embedded_doc: false,
            is_swift,
            lua_block_level: 0,
            in_dlang_nesting_block: false,
            dlang_nesting_depth: 0,
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
                    // Rust/Kotlin/Scala などのネストコメント対応
                    process_nesting_c_style_with_options(
                        trimmed,
                        &self.string_opts,
                        &mut self.block_comment_depth,
                        &mut self.in_block_comment,
                        &mut self.count,
                    );
                } else {
                    // C, C++, Java, Go, JS/TS など - 言語別の文字列オプションを使用
                    // (C++ Raw String は StringSkipOptions::cpp() に含まれる)
                    process_c_style_with_options(
                        trimmed,
                        &self.string_opts,
                        &mut self.in_block_comment,
                        &mut self.count,
                    );
                }
            }
            CommentStyle::Python => {
                // Python: Docstring ("""/''') と f-string 対応
                process_python_style(
                    line, // Docstringは行頭判定が必要なため trim 前の line を渡す
                    &mut self.docstring_quote,
                    &mut self.in_block_comment,
                    &mut self.count,
                );
            }
            CommentStyle::Ruby => {
                // Ruby: =begin/=end 埋め込みドキュメント対応
                process_ruby_style(
                    line, // 埋め込みドキュメントは行頭判定が必要
                    &mut self.in_embedded_doc,
                    &mut self.count,
                );
            }
            CommentStyle::Perl => {
                // Perl: POD (=pod/=head/=cut) 対応
                process_perl_style(
                    line, // PODは行頭判定が必要
                    &mut self.in_embedded_doc,
                    &mut self.count,
                );
            }
            CommentStyle::SimpleHash => {
                // Shell, YAML, Config系など: 単純な # コメント
                process_simple_hash_style(trimmed, &mut self.count);
            }
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
            CommentStyle::DLang => {
                process_dlang_style(
                    trimmed,
                    &mut self.in_block_comment,
                    &mut self.in_dlang_nesting_block,
                    &mut self.dlang_nesting_depth,
                    &mut self.count,
                )
            }
            CommentStyle::Batch => process_batch_style(trimmed, &mut self.count),
            CommentStyle::Assembly => process_assembly_style(trimmed, &mut self.count),
            CommentStyle::GasAssembly => {
                process_gas_assembly_style(trimmed, &mut self.in_block_comment, &mut self.count)
            }
            CommentStyle::Vhdl => process_vhdl_style(trimmed, &mut self.count),
            CommentStyle::VisualBasic => process_visual_basic_style(trimmed, &mut self.count),
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

    // ==================== C# Verbatim String テスト ====================

    #[test]
    fn test_csharp_verbatim_string_basic() {
        let mut counter = SlocCounter::new("cs");
        counter.process_line(r#"var path = @"C:\MyFolder\file.txt";"#);
        counter.process_line("var x = 1;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_csharp_verbatim_string_with_comment_marker() {
        let mut counter = SlocCounter::new("cs");
        counter.process_line(r#"var regex = @"^# not a comment$";"#);
        counter.process_line("var y = 2;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_csharp_verbatim_string_escaped_quote() {
        let mut counter = SlocCounter::new("cs");
        // "" は " 一文字にエスケープされる
        counter.process_line(r#"var s = @"Quotes""Here""";"#);
        counter.process_line("var z = 3;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_csharp_verbatim_string_with_block_comment_marker() {
        let mut counter = SlocCounter::new("cs");
        counter.process_line(r#"var s = @"/* not a comment */";"#);
        counter.process_line("var w = 4;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    // ==================== Java/Kotlin Text Block テスト ====================

    #[test]
    fn test_java_text_block_basic() {
        let mut counter = SlocCounter::new("java");
        counter.process_line(r#"String s = """text block""";"#);
        counter.process_line("int x = 1;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_java_text_block_with_comment_marker() {
        let mut counter = SlocCounter::new("java");
        counter.process_line(r#"String s = """// not a comment""";"#);
        counter.process_line("int y = 2;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_kotlin_text_block_with_block_comment_marker() {
        let mut counter = SlocCounter::new("kt");
        counter.process_line(r#"val s = """/* not a comment */""";"#);
        counter.process_line("val z = 3;");
        // 両方の行がSLOC
        assert_eq!(counter.count(), 2);
    }

    // ==================== SQL 文字列内コメントマーカー テスト ====================

    #[test]
    fn test_sql_string_with_block_comment_marker() {
        let mut counter = SlocCounter::new("sql");
        counter.process_line("SELECT '/* これはコメントではありません */' FROM users;");
        // 1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_sql_string_with_line_comment_marker() {
        let mut counter = SlocCounter::new("sql");
        counter.process_line("SELECT '-- これもコメントではない' FROM users;");
        // 1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_sql_escaped_quote() {
        let mut counter = SlocCounter::new("sql");
        // '' は ' 1文字にエスケープ
        counter.process_line("SELECT 'It''s a test /* not comment */' FROM t;");
        // 1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_sql_double_quote_identifier() {
        let mut counter = SlocCounter::new("sql");
        counter.process_line(r#"SELECT "column /* name */" FROM t;"#);
        // 1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_sql_real_block_comment() {
        let mut counter = SlocCounter::new("sql");
        counter.process_line("SELECT * /* comment */ FROM t;");
        // 1行がSLOC (コメント前後にコードがある)
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_sql_line_comment_after_code() {
        let mut counter = SlocCounter::new("sql");
        counter.process_line("SELECT * FROM t; -- comment");
        // 1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== D言語 テスト ====================

    #[test]
    fn test_dlang_line_comment() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("// comment");
        counter.process_line("int x = 1;");
        counter.process_line("int y = 2; // inline comment");
        // x と y の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_dlang_block_comment() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/* block comment */");
        counter.process_line("int z = 3;");
        // z の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_nesting_comment_basic() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/+");
        counter.process_line("  nesting comment");
        counter.process_line("+/");
        counter.process_line("int a = 1;");
        // a の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_nesting_comment_nested() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/+ outer");
        counter.process_line("/+ inner +/");
        counter.process_line("still in outer +/");
        counter.process_line("int b = 2;");
        // b の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_nesting_comment_single_line() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/+ /+ nested +/ still in outer +/ int c = 3;");
        // c の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_nesting_comment_deep() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/+ level 1");
        counter.process_line("/+ level 2");
        counter.process_line("/+ level 3 +/");
        counter.process_line("back to level 2 +/");
        counter.process_line("back to level 1 +/");
        counter.process_line("int d = 4;");
        // d の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_mixed_comments() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/* block */ /+ nesting +/ int x = 1;");
        // x の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_code_before_nesting_comment() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("int x = 1; /+ comment");
        counter.process_line("/+ nested +/");
        counter.process_line("+/");
        counter.process_line("int y = 2;");
        // x と y の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    // ==================== Windows バッチファイル テスト ====================

    #[test]
    fn test_batch_rem_comment() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line("REM This is a comment");
        counter.process_line("echo Hello");
        // echo の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_rem_lowercase() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line("rem lowercase comment");
        counter.process_line("set x=1");
        // set の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_double_colon_comment() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line(":: This is a label comment");
        counter.process_line("echo World");
        // echo の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_at_rem() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line("@REM Suppress output and comment");
        counter.process_line("@echo off");
        // @echo の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_cmd_extension() {
        let mut counter = SlocCounter::new("cmd");
        counter.process_line(":: CMD file comment");
        counter.process_line("dir /w");
        // dir の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_rem_only() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line("REM");
        counter.process_line("echo test");
        // echo の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_not_rem_if_no_space() {
        let mut counter = SlocCounter::new("bat");
        // "REMARK" は REM コメントではない
        counter.process_line("echo REMARK");
        // echo の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_batch_rem_with_tab() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line("REM\tcomment with tab");
        counter.process_line("set y=2");
        // set の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== Assembly (NASM/MASM) テスト ====================

    #[test]
    fn test_assembly_nasm_line_comment() {
        let mut counter = SlocCounter::new("asm");
        counter.process_line("; This is a NASM comment");
        counter.process_line("mov eax, 1");
        // mov の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_assembly_inline_comment() {
        let mut counter = SlocCounter::new("asm");
        counter.process_line("mov eax, ebx ; copy ebx to eax");
        counter.process_line("ret");
        // mov と ret の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_assembly_multiple_lines() {
        let mut counter = SlocCounter::new("asm");
        counter.process_line("; Function prologue");
        counter.process_line("push ebp");
        counter.process_line("mov ebp, esp");
        counter.process_line("; Save registers");
        counter.process_line("push ebx");
        // 3行がSLOC (push, mov, push)
        assert_eq!(counter.count(), 3);
    }

    // ==================== GAS Assembly テスト ====================

    #[test]
    fn test_gas_assembly_hash_comment() {
        let mut counter = SlocCounter::new("s");
        counter.process_line("# GAS comment");
        counter.process_line("movl $1, %eax");
        // movl の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_gas_assembly_inline_comment() {
        let mut counter = SlocCounter::new("s");
        counter.process_line("movl %ebx, %eax # copy");
        counter.process_line("ret");
        // 2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_gas_assembly_block_comment() {
        let mut counter = SlocCounter::new("s");
        counter.process_line("/* block comment */");
        counter.process_line("movl $0, %eax");
        // movl の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_gas_assembly_multiline_block_comment() {
        let mut counter = SlocCounter::new("s");
        counter.process_line("/*");
        counter.process_line("  multiline comment");
        counter.process_line("*/");
        counter.process_line("ret");
        // ret の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_gas_assembly_at_comment() {
        // ARM GAS では @ がコメント
        let mut counter = SlocCounter::new("s");
        counter.process_line("@ ARM comment");
        counter.process_line("mov r0, #1");
        // mov の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== VHDL テスト ====================

    #[test]
    fn test_vhdl_line_comment() {
        let mut counter = SlocCounter::new("vhd");
        counter.process_line("-- VHDL comment");
        counter.process_line("signal clk : std_logic;");
        // signal の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_vhdl_inline_comment() {
        let mut counter = SlocCounter::new("vhdl");
        counter.process_line("signal rst : std_logic; -- reset signal");
        counter.process_line("signal data : std_logic_vector(7 downto 0);");
        // 2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_vhdl_entity() {
        let mut counter = SlocCounter::new("vhd");
        counter.process_line("-- Entity declaration");
        counter.process_line("entity counter is");
        counter.process_line("port (");
        counter.process_line("    clk : in std_logic; -- clock input");
        counter.process_line("    -- rst : in std_logic;");
        counter.process_line("    count : out std_logic_vector(7 downto 0)");
        counter.process_line(");");
        counter.process_line("end entity;");
        // 5行がSLOC (entity, port, clk, count, ), end)
        assert_eq!(counter.count(), 6);
    }

    // ==================== Verilog/SystemVerilog テスト ====================

    #[test]
    fn test_verilog_line_comment() {
        let mut counter = SlocCounter::new("sv");
        counter.process_line("// SystemVerilog comment");
        counter.process_line("wire clk;");
        // wire の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_verilog_block_comment() {
        let mut counter = SlocCounter::new("sv");
        counter.process_line("/* block comment */");
        counter.process_line("reg [7:0] data;");
        // reg の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_systemverilog_header() {
        let mut counter = SlocCounter::new("svh");
        counter.process_line("// Header file");
        counter.process_line("`define WIDTH 8");
        // `define の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== リンカスクリプト テスト ====================

    #[test]
    fn test_linker_script_comment() {
        let mut counter = SlocCounter::new("ld");
        counter.process_line("/* Linker script */");
        counter.process_line("ENTRY(_start)");
        // ENTRY の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_linker_script_multiline() {
        let mut counter = SlocCounter::new("lds");
        counter.process_line("/*");
        counter.process_line(" * Memory layout");
        counter.process_line(" */");
        counter.process_line("MEMORY {");
        counter.process_line("    ROM : ORIGIN = 0x0, LENGTH = 64K");
        counter.process_line("}");
        // MEMORY, ROM, } の3行がSLOC
        assert_eq!(counter.count(), 3);
    }

    // ==================== LaTeX テスト ====================

    #[test]
    fn test_latex_percent_comment() {
        let mut counter = SlocCounter::new("tex");
        counter.process_line("% This is a LaTeX comment");
        counter.process_line("\\documentclass{article}");
        // \\documentclass の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_latex_inline_comment() {
        let mut counter = SlocCounter::new("tex");
        counter.process_line("\\begin{document} % Start document");
        // % 以降はコメントだが、その前にコードがある → 現在の実装では starts_with('%') のみ
        // LaTeX は Erlang スタイルで starts_with('%') のみチェックなので、この行はSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_latex_style_file() {
        let mut counter = SlocCounter::new("sty");
        counter.process_line("% Package definition");
        counter.process_line("\\ProvidesPackage{mypackage}");
        // \\ProvidesPackage の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_latex_bib_file() {
        let mut counter = SlocCounter::new("bib");
        counter.process_line("% Bibliography");
        counter.process_line("@article{key,");
        counter.process_line("  author = {Author},");
        counter.process_line("}");
        // @article, author, } の3行がSLOC
        assert_eq!(counter.count(), 3);
    }

    // ==================== 設定ファイル テスト ====================

    #[test]
    fn test_ini_hash_comment() {
        let mut counter = SlocCounter::new("ini");
        counter.process_line("# INI comment");
        counter.process_line("[section]");
        counter.process_line("key = value");
        // [section] と key の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_conf_file() {
        let mut counter = SlocCounter::new("conf");
        counter.process_line("# Configuration");
        counter.process_line("server = localhost");
        counter.process_line("port = 8080");
        // server と port の2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_properties_file() {
        let mut counter = SlocCounter::new("properties");
        counter.process_line("# Java properties");
        counter.process_line("app.name=MyApp");
        counter.process_line("app.version=1.0");
        // 2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    // ==================== Visual Basic テスト ====================

    #[test]
    fn test_vb_single_quote_comment() {
        let mut counter = SlocCounter::new("vb");
        counter.process_line("' This is a VB comment");
        counter.process_line("Dim x As Integer");
        // Dim の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_vb_rem_comment() {
        let mut counter = SlocCounter::new("vb");
        counter.process_line("REM This is a REM comment");
        counter.process_line("x = 10");
        // x = 10 の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_vb_inline_comment() {
        let mut counter = SlocCounter::new("vb");
        counter.process_line("Dim y As String ' variable declaration");
        counter.process_line("y = \"Hello\"");
        // 2行がSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_vb_string_with_quote() {
        let mut counter = SlocCounter::new("vb");
        counter.process_line("s = \"It's a test\" ' comment");
        // 1行がSLOC（文字列内の ' はコメントではない）
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_vba_class_module() {
        let mut counter = SlocCounter::new("cls");
        counter.process_line("' Class module");
        counter.process_line("Private m_value As Long");
        counter.process_line("REM Property getter");
        counter.process_line("Public Property Get Value() As Long");
        counter.process_line("    Value = m_value ' return value");
        counter.process_line("End Property");
        // 4行がSLOC (Private, Public, Value, End)
        assert_eq!(counter.count(), 4);
    }

    #[test]
    fn test_vbs_script() {
        let mut counter = SlocCounter::new("vbs");
        counter.process_line("' VBScript");
        counter.process_line("WScript.Echo \"Hello, World!\"");
        // 1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    // ==================== Protocol Buffers テスト ====================

    #[test]
    fn test_protobuf_line_comment() {
        let mut counter = SlocCounter::new("proto");
        counter.process_line("// Protocol buffer definition");
        counter.process_line("syntax = \"proto3\";");
        // syntax の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_protobuf_message() {
        let mut counter = SlocCounter::new("proto");
        counter.process_line("message Person {");
        counter.process_line("  // Name field");
        counter.process_line("  string name = 1;");
        counter.process_line("  /* age field */");
        counter.process_line("  int32 age = 2;");
        counter.process_line("}");
        // 4行がSLOC (message, name, age, })
        assert_eq!(counter.count(), 4);
    }

    // ==================== GraphQL テスト ====================

    #[test]
    fn test_graphql_hash_comment() {
        let mut counter = SlocCounter::new("graphql");
        counter.process_line("# GraphQL schema");
        counter.process_line("type Query {");
        counter.process_line("  users: [User]");
        counter.process_line("}");
        // 3行がSLOC (type, users, })
        assert_eq!(counter.count(), 3);
    }

    #[test]
    fn test_graphql_gql_extension() {
        let mut counter = SlocCounter::new("gql");
        counter.process_line("# Mutation");
        counter.process_line("type Mutation {");
        counter.process_line("  createUser(name: String!): User");
        counter.process_line("}");
        // 3行がSLOC
        assert_eq!(counter.count(), 3);
    }

    // ==================== Solidity テスト ====================

    #[test]
    fn test_solidity_line_comment() {
        let mut counter = SlocCounter::new("sol");
        counter.process_line("// SPDX-License-Identifier: MIT");
        counter.process_line("pragma solidity ^0.8.0;");
        // pragma の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_solidity_contract() {
        let mut counter = SlocCounter::new("sol");
        counter.process_line("/* ERC20 Token */");
        counter.process_line("contract Token {");
        counter.process_line("    uint256 public totalSupply; // total supply");
        counter.process_line("}");
        // 3行がSLOC (contract, totalSupply, })
        assert_eq!(counter.count(), 3);
    }

    // ==================== Thrift テスト ====================

    #[test]
    fn test_thrift_line_comment() {
        let mut counter = SlocCounter::new("thrift");
        counter.process_line("// Thrift IDL");
        counter.process_line("namespace java com.example");
        // namespace の1行がSLOC
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_thrift_struct() {
        let mut counter = SlocCounter::new("thrift");
        counter.process_line("struct User {");
        counter.process_line("  /* user id */");
        counter.process_line("  1: i64 id,");
        counter.process_line("  2: string name, // user name");
        counter.process_line("}");
        // 4行がSLOC (struct, id, name, })
        assert_eq!(counter.count(), 4);
    }
}
