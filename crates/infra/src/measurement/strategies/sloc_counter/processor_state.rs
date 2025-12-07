// crates/infra/src/measurement/strategies/sloc_counter/processor_state.rs
//! プロセッサ状態管理
//!
//! 各言語のコメント処理ロジックと状態を SlocProcessor Enum で管理し、
//! Enum Dispatch パターンで適切なプロセッサに処理を委譲します。

use super::comment_style::CommentStyle;
use super::processors::*;
use super::string_utils::StringSkipOptions;

// ============================================================================
// SlocProcessor Enum
// ============================================================================

/// SLOCプロセッサ (Enum Dispatch)
///
/// 各言語ごとの処理ロジックと状態を保持します。
/// これにより、SlocCounter から言語固有の状態を分離し、
/// メモリ効率と型安全性を向上させます。
pub enum SlocProcessor {
    /// C系言語 (//, /* */) - ネストなし
    CStyle(CStyleProcessor),
    /// C系言語 (//, /* */) - ネスト対応 (Rust, Kotlin, Scala)
    NestingCStyle(NestingCStyleProcessor),
    /// Swift (拡張デリミタ文字列対応 + ネストコメント)
    Swift(SwiftProcessor),
    /// Python (Docstring, f-string)
    Python(PythonProcessor),
    /// Ruby (=begin/=end)
    Ruby(RubyProcessor),
    /// Perl (POD)
    Perl(PerlProcessor),
    /// PHP (//, /* */, #)
    Php(PhpProcessor),
    /// PowerShell (# と <# #>)
    PowerShell(PowerShellProcessor),
    /// Lua (-- と --[[ ]])
    Lua(LuaProcessor),
    /// HTML/XML (<!-- -->)
    Html(HtmlProcessor),
    /// SQL (-- と /* */)
    Sql(SqlProcessor),
    /// Haskell (-- と {- -})
    Haskell(HaskellProcessor),
    /// Julia (# と #= =#)
    Julia(JuliaProcessor),
    /// OCaml/F#/Pascal ((* *))
    OCaml(OCamlProcessor),
    /// D言語 (//, /* */, /+ +/)
    DLang(DLangProcessor),
    /// MATLAB/Octave (% と %{ %})
    Matlab(MatlabProcessor),
    /// GAS Assembly (# と /* */)
    GasAssembly(GasAssemblyProcessor),
    /// 単純な行コメント (#) (Shell, YAML, etc.)
    SimpleHash(SimpleHashProcessor),
    /// 単純なプレフィックス型コメント (VHDL, Erlang, Lisp, Batch, etc.)
    SimplePrefix(SimplePrefixProcessor),
    /// コメントなし
    NoComment,
}

// ============================================================================
// Processor structs: すべて processors/ ディレクトリの各ファイルで定義済み
// ============================================================================

// ============================================================================
// SlocProcessor Implementation
// ============================================================================

impl SlocProcessor {
    /// 拡張子からプロセッサを作成
    pub fn from_extension(extension: &str) -> Self {
        let style = CommentStyle::from_extension(extension);
        let ext_lower = extension.to_lowercase();
        let string_opts = StringSkipOptions::from_extension(extension);

        match style {
            CommentStyle::CStyle => {
                // Swift は拡張デリミタ文字列対応の専用処理を使用
                if ext_lower == "swift" {
                    Self::Swift(SwiftProcessor::new())
                } else if matches!(ext_lower.as_str(), "rs" | "kt" | "kts" | "scala" | "sc") {
                    // Rust/Kotlin/Scala はネスト対応
                    Self::NestingCStyle(NestingCStyleProcessor::new(string_opts))
                } else {
                    Self::CStyle(CStyleProcessor::new(string_opts))
                }
            }
            CommentStyle::Python => Self::Python(PythonProcessor::default()),
            CommentStyle::Ruby => Self::Ruby(RubyProcessor::default()),
            CommentStyle::Perl => Self::Perl(PerlProcessor::default()),
            CommentStyle::Php => Self::Php(PhpProcessor::new()),
            CommentStyle::PowerShell => Self::PowerShell(PowerShellProcessor::new()),
            CommentStyle::Lua => Self::Lua(LuaProcessor::new()),
            CommentStyle::Html => Self::Html(HtmlProcessor::new()),
            CommentStyle::Sql => Self::Sql(SqlProcessor::new()),
            CommentStyle::Haskell => Self::Haskell(HaskellProcessor::new()),
            CommentStyle::Julia => Self::Julia(JuliaProcessor::new()),
            CommentStyle::OCaml => Self::OCaml(OCamlProcessor::new()),
            CommentStyle::DLang => Self::DLang(DLangProcessor::new()),
            CommentStyle::Matlab => Self::Matlab(MatlabProcessor::new()),
            CommentStyle::GasAssembly => Self::GasAssembly(GasAssemblyProcessor::new()),
            CommentStyle::SimpleHash => Self::SimpleHash(SimpleHashProcessor::default()),
            // 単純なプレフィックス型言語は SimplePrefixProcessor に統合
            CommentStyle::Vhdl => Self::SimplePrefix(SimplePrefixProcessor::vhdl()),
            CommentStyle::Erlang => Self::SimplePrefix(SimplePrefixProcessor::erlang()),
            CommentStyle::Lisp => Self::SimplePrefix(SimplePrefixProcessor::lisp()),
            CommentStyle::Assembly => Self::SimplePrefix(SimplePrefixProcessor::assembly()),
            CommentStyle::Fortran => Self::SimplePrefix(SimplePrefixProcessor::fortran()),
            CommentStyle::Batch => Self::SimplePrefix(SimplePrefixProcessor::batch()),
            CommentStyle::VisualBasic => Self::SimplePrefix(SimplePrefixProcessor::visual_basic()),
            CommentStyle::None => Self::NoComment,
        }
    }

    /// 行を処理し、SLOCとしてカウントすべきか（1 or 0）を返す
    pub fn process_line(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        match self {
            Self::CStyle(p) => p.process(trimmed),
            Self::NestingCStyle(p) => p.process(trimmed),
            Self::Swift(p) => p.process(trimmed),
            Self::Python(p) => p.process(line), // Docstringで行頭判定が必要
            Self::Ruby(p) => p.process(line),   // 埋め込みドキュメントで行頭判定が必要
            Self::Perl(p) => p.process(line),   // PODで行頭判定が必要
            Self::Php(p) => p.process(trimmed),
            Self::PowerShell(p) => p.process(trimmed),
            Self::Lua(p) => p.process(trimmed),
            Self::Html(p) => p.process(trimmed),
            Self::Sql(p) => p.process(trimmed),
            Self::Haskell(p) => p.process(trimmed),
            Self::Julia(p) => p.process(trimmed),
            Self::OCaml(p) => p.process(trimmed),
            Self::DLang(p) => p.process(trimmed),
            Self::Matlab(p) => p.process(trimmed),
            Self::GasAssembly(p) => p.process(trimmed),
            Self::SimpleHash(p) => p.process(line),
            Self::SimplePrefix(p) => p.process(trimmed),
            Self::NoComment => 1, // 非空行は全てSLOC
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_c_style() {
        let mut processor = SlocProcessor::from_extension("c");
        assert_eq!(processor.process_line("// comment"), 0);
        assert_eq!(processor.process_line("int x = 1;"), 1);
    }

    #[test]
    fn test_processor_rust() {
        let mut processor = SlocProcessor::from_extension("rs");
        assert_eq!(processor.process_line("/* outer"), 0);
        assert_eq!(processor.process_line("/* nested */"), 0);
        assert_eq!(processor.process_line("*/"), 0);
        assert_eq!(processor.process_line("let x = 1;"), 1);
    }

    #[test]
    fn test_processor_python() {
        let mut processor = SlocProcessor::from_extension("py");
        assert_eq!(processor.process_line("# comment"), 0);
        assert_eq!(processor.process_line("x = 1"), 1);
    }

    #[test]
    fn test_processor_shell() {
        let mut processor = SlocProcessor::from_extension("sh");
        assert_eq!(processor.process_line("#!/bin/bash"), 0);
        assert_eq!(processor.process_line("# comment"), 0);
        assert_eq!(processor.process_line("echo hello"), 1);
    }

    // SimplePrefixProcessor 統合テスト
    #[test]
    fn test_processor_vhdl() {
        let mut processor = SlocProcessor::from_extension("vhdl");
        assert_eq!(processor.process_line("-- comment"), 0);
        assert_eq!(processor.process_line("signal x : integer;"), 1);
    }

    #[test]
    fn test_processor_erlang() {
        let mut processor = SlocProcessor::from_extension("erl");
        assert_eq!(processor.process_line("% comment"), 0);
        assert_eq!(processor.process_line("-module(test)."), 1);
    }

    #[test]
    fn test_processor_lisp() {
        let mut processor = SlocProcessor::from_extension("lisp");
        assert_eq!(processor.process_line("; comment"), 0);
        assert_eq!(processor.process_line("(defun foo () 1)"), 1);
    }

    #[test]
    fn test_processor_batch() {
        let mut processor = SlocProcessor::from_extension("bat");
        assert_eq!(processor.process_line("REM comment"), 0);
        assert_eq!(processor.process_line(":: label comment"), 0);
        assert_eq!(processor.process_line("echo hello"), 1);
    }

    #[test]
    fn test_processor_fortran() {
        let mut processor = SlocProcessor::from_extension("f90");
        assert_eq!(processor.process_line("! comment"), 0);
        assert_eq!(processor.process_line("program test"), 1);
    }
}
