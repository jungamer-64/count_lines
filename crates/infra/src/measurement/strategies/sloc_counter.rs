// crates/infra/src/measurement/strategies/sloc_counter.rs
//! SLOC (Source Lines of Code) カウンター
//!
//! 言語ごとのコメント構文を認識し、純粋なコード行のみをカウントします。
//!
//! ## 設計
//!
//! このモジュールは Enum Dispatch パターンを使用して、言語固有のコメント処理を
//! 各プロセッサに委譲します。これにより、`SlocCounter` 自体は言語非依存となり、
//! 言語固有の状態（ブロックコメント深度、Docstring状態など）は各プロセッサ内で管理されます。

mod comment_style;
mod processor_state;
mod processor_trait;
pub mod processors;
mod string_utils;

pub use comment_style::CommentStyle;
use processor_state::SlocProcessor;
pub use processor_trait::LineProcessor;

/// SLOCカウンター
///
/// 言語ごとのコメント構文を認識し、純粋なコード行のみをカウントします。
///
/// # 使用例
///
/// ```ignore
/// let mut counter = SlocCounter::new("rs");
/// counter.process_line("// this is a comment");
/// counter.process_line("let x = 1;");
/// assert_eq!(counter.count(), 1);
/// ```
pub struct SlocCounter {
    /// 言語固有のプロセッサ (Enum Dispatch)
    processor: SlocProcessor,
    /// 現在のSLOCカウント
    count: usize,
}

impl SlocCounter {
    /// 新しいカウンターを作成
    ///
    /// # Arguments
    ///
    /// * `extension` - ファイル拡張子（例: "rs", "py", "cpp"）
    pub fn new(extension: &str) -> Self {
        let processor = SlocProcessor::from_extension(extension);
        Self {
            processor,
            count: 0,
        }
    }

    /// 行を処理してSLOCかどうかを判定
    ///
    /// # Arguments
    ///
    /// * `line` - 処理する行
    pub fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();

        // 空行はスキップ
        if trimmed.is_empty() {
            return;
        }

        self.count += self.processor.process_line(line);
    }

    /// 現在のSLOCカウントを取得
    pub fn count(&self) -> usize {
        self.count
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
        ];
        for line in lines {
            counter.process_line(line);
        }
        // SLOC: use (2) + #[derive] (1) + pub enum (1) + variants (2) + } (1) = 7
        assert_eq!(counter.count(), 7);
    }

    #[test]
    fn test_c_style_block_comment() {
        let mut counter = SlocCounter::new("c");
        counter.process_line("/*");
        counter.process_line(" * block comment");
        counter.process_line(" */");
        counter.process_line("int x = 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_c_style_inline_block_comment() {
        let mut counter = SlocCounter::new("c");
        counter.process_line("int x = /* comment */ 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_rust_nested_block_comment() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("/* outer /* inner */ still comment */");
        counter.process_line("let x = 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_python_style() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("# comment");
        counter.process_line("x = 1  # inline comment");
        counter.process_line("y = 2");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_python_docstring() {
        let mut counter = SlocCounter::new("py");
        counter.process_line("def foo():");
        counter.process_line("    \"\"\"");
        counter.process_line("    Docstring");
        counter.process_line("    \"\"\"");
        counter.process_line("    return 1");
        // def foo(): と return 1 のみがSLOC
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_shell_style() {
        let mut counter = SlocCounter::new("sh");
        counter.process_line("#!/bin/bash");
        counter.process_line("# comment");
        counter.process_line("echo 'hello'");
        counter.process_line("exit 0");
        // shebang と # comment は除外
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_ruby_embedded_doc() {
        let mut counter = SlocCounter::new("rb");
        counter.process_line("x = 1");
        counter.process_line("=begin");
        counter.process_line("embedded doc");
        counter.process_line("=end");
        counter.process_line("y = 2");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_perl_pod() {
        let mut counter = SlocCounter::new("pl");
        counter.process_line("use strict;");
        counter.process_line("=head1 NAME");
        counter.process_line("MyModule");
        counter.process_line("=cut");
        counter.process_line("print 1;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_lua_block_comment() {
        let mut counter = SlocCounter::new("lua");
        counter.process_line("-- line comment");
        counter.process_line("--[[");
        counter.process_line("block comment");
        counter.process_line("]]");
        counter.process_line("local x = 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_html_comment() {
        let mut counter = SlocCounter::new("html");
        counter.process_line("<!-- comment -->");
        counter.process_line("<html>");
        counter.process_line("</html>");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_sql_comment() {
        let mut counter = SlocCounter::new("sql");
        counter.process_line("-- comment");
        counter.process_line("/* block */");
        counter.process_line("SELECT * FROM users;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_no_comment_style() {
        let mut counter = SlocCounter::new("txt");
        counter.process_line("line 1");
        counter.process_line("");
        counter.process_line("line 2");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_empty_file() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line("");
        counter.process_line("   ");
        counter.process_line("\t");
        assert_eq!(counter.count(), 0);
    }

    #[test]
    fn test_string_with_comment_marker() {
        let mut counter = SlocCounter::new("rs");
        counter.process_line(r#"let s = "// not a comment";"#);
        counter.process_line(r#"let t = "/* also not a comment */";"#);
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_cpp_raw_string() {
        let mut counter = SlocCounter::new("cpp");
        counter.process_line(r#"const char* s = R"(/* not a comment */)";"#);
        counter.process_line("int x = 1;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_kotlin_nested_comments() {
        let mut counter = SlocCounter::new("kt");
        counter.process_line("/* outer /* inner */ still comment */");
        counter.process_line("val x = 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_go_backtick_string() {
        let mut counter = SlocCounter::new("go");
        counter.process_line("s := `/* not a comment */`");
        counter.process_line("x := 1");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_javascript_template_literal() {
        let mut counter = SlocCounter::new("js");
        counter.process_line("const s = `// not a comment`;");
        counter.process_line("const x = 1;");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_yaml_inline_comment() {
        let mut counter = SlocCounter::new("yaml");
        counter.process_line("# comment");
        counter.process_line("name: value  # inline");
        counter.process_line("another: value");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_php_multiple_comment_styles() {
        let mut counter = SlocCounter::new("php");
        counter.process_line("<?php");
        counter.process_line("// line comment");
        counter.process_line("# hash comment");
        counter.process_line("/* block comment */");
        counter.process_line("echo 'hello';");
        counter.process_line("?>");
        assert_eq!(counter.count(), 3); // <?php, echo, ?>
    }

    #[test]
    fn test_powershell_block_comment() {
        let mut counter = SlocCounter::new("ps1");
        counter.process_line("# line comment");
        counter.process_line("<#");
        counter.process_line("block comment");
        counter.process_line("#>");
        counter.process_line("Write-Host 'hello'");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_haskell_nested_comment() {
        let mut counter = SlocCounter::new("hs");
        counter.process_line("-- line comment");
        counter.process_line("{- block {- nested -} comment -}");
        counter.process_line("main = putStrLn \"hello\"");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_lisp_comment() {
        let mut counter = SlocCounter::new("lisp");
        counter.process_line("; comment");
        counter.process_line("(defun foo () 1)");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_erlang_comment() {
        let mut counter = SlocCounter::new("erl");
        counter.process_line("% comment");
        counter.process_line("-module(test).");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_fortran_comment() {
        let mut counter = SlocCounter::new("f90");
        counter.process_line("! comment");
        counter.process_line("program test");
        counter.process_line("end program");
        assert_eq!(counter.count(), 2);
    }

    #[test]
    fn test_batch_comment() {
        let mut counter = SlocCounter::new("bat");
        counter.process_line("REM comment");
        counter.process_line(":: another comment");
        counter.process_line("echo hello");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_assembly_comment() {
        let mut counter = SlocCounter::new("asm");
        counter.process_line("; comment");
        counter.process_line("mov ax, 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_vhdl_comment() {
        let mut counter = SlocCounter::new("vhdl");
        counter.process_line("-- comment");
        counter.process_line("entity test is");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_visual_basic_comment() {
        let mut counter = SlocCounter::new("vb");
        counter.process_line("' comment");
        counter.process_line("REM another comment");
        counter.process_line("Dim x As Integer");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_matlab_block_comment() {
        let mut counter = SlocCounter::new("mat");
        counter.process_line("% line comment");
        counter.process_line("%{");
        counter.process_line("block comment");
        counter.process_line("%}");
        counter.process_line("x = 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_julia_block_comment() {
        let mut counter = SlocCounter::new("jl");
        counter.process_line("# line comment");
        counter.process_line("#=");
        counter.process_line("block comment");
        counter.process_line("=#");
        counter.process_line("x = 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_ocaml_nested_comment() {
        let mut counter = SlocCounter::new("ml");
        counter.process_line("(* outer (* nested *) comment *)");
        counter.process_line("let x = 1");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_dlang_nesting_comment() {
        let mut counter = SlocCounter::new("d");
        counter.process_line("/+ outer /+ nested +/ comment +/");
        counter.process_line("int x = 1;");
        assert_eq!(counter.count(), 1);
    }

    #[test]
    fn test_gas_assembly() {
        let mut counter = SlocCounter::new("s");
        counter.process_line("# comment");
        counter.process_line("/* block comment */");
        counter.process_line("movl $1, %eax");
        assert_eq!(counter.count(), 1);
    }
}
