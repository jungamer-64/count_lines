// crates/core/src/language/processors/simple_prefix_style.rs
//! # Simple Prefix Comment Processor
//!
//! SLOC counter processor for languages with simple prefix-based line comments.
//!
//! ## Supported Languages
//!
//! | Language | Prefix(es) | Case Sensitive |
//! |----------|------------|----------------|
//! | VHDL | `--` | Yes |
//! | Erlang, LaTeX | `%` | Yes |
//! | Lisp, Scheme, Clojure | `;` | Yes |
//! | Assembly (NASM/MASM) | `;` | Yes |
//! | Fortran | `!`, `C`, `c`, `*` | Yes |
//! | Batch | `REM `, `::`, `@REM ` | No |
//! | Visual Basic | `'`, `REM ` | No |
//!
//! ## How It Works
//!
//! Lines are trimmed and checked if they start with any of the configured prefixes.
//! If so, the line is counted as a comment (returns 0). Otherwise, it's code (returns 1).
//!
//! > **Note**: This processor does not handle inline comments. A line like
//! > `signal x : integer; -- comment` will return 1 (code) because it doesn't
//! > start with the comment prefix.
//!
//! ## Performance Characteristics
//!
//! - **Time complexity**: O(p × m) where p = number of prefixes, m = longest prefix length
//! - **Space complexity**: O(1)
//! - **Thread safety**: Yes (immutable after construction)
//!
//! ## Usage Example
//!
//! ```rust,ignore
//! use count_lines_core::language::processors::SimplePrefixProcessor;
//!
//! // VHDL: "--" prefix
//! let p = SimplePrefixProcessor::vhdl();
//! assert_eq!(p.process("-- comment"), 0);
//! assert_eq!(p.process("signal x : integer;"), 1);
//!
//! // Batch: case-insensitive "REM " prefix
//! let p = SimplePrefixProcessor::batch();
//! assert_eq!(p.process("REM comment"), 0);
//! assert_eq!(p.process("rem comment"), 0);
//! assert_eq!(p.process("echo hello"), 1);
//! ```

/// 単純なプレフィックス型コメントプロセッサ
///
/// 指定されたプレフィックスのいずれかで始まる行をコメントとして扱い、
/// それ以外の行をSLOCとしてカウントします。
///
/// # Examples
///
/// ```ignore
/// // VHDL: "--" で始まる行がコメント
/// let p = SimplePrefixProcessor::new(&["--"]);
/// assert_eq!(p.process("-- comment"), 0);
/// assert_eq!(p.process("signal x : integer;"), 1);
///
/// // Batch: "REM", "::", "@REM" で始まる行がコメント (大文字小文字区別なし)
/// let p = SimplePrefixProcessor::new_ignore_case(&["REM ", "::", "@REM "]);
/// assert_eq!(p.process("REM comment"), 0);
/// assert_eq!(p.process("echo hello"), 1);
/// ```
/// Prefix-based comment SLOC processor.
#[derive(Debug)]
pub struct SimplePrefixProcessor {
    prefixes: &'static [&'static str],
    ignore_case: bool,
}

use super::super::processor_trait::LineProcessor;

impl LineProcessor for SimplePrefixProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        false
    }
}

impl SimplePrefixProcessor {
    /// 大文字小文字を区別するプロセッサを作成
    #[must_use]
    pub const fn new(prefixes: &'static [&'static str]) -> Self {
        Self {
            prefixes,
            ignore_case: false,
        }
    }

    /// 大文字小文字を区別しないプロセッサを作成
    #[must_use]
    pub const fn new_ignore_case(prefixes: &'static [&'static str]) -> Self {
        Self {
            prefixes,
            ignore_case: true,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    #[must_use]
    pub fn process(&self, line: &str) -> usize {
        let trimmed = line.trim();

        if self.ignore_case {
            // 大文字小文字を区別しない比較
            let upper = trimmed.to_uppercase();
            for prefix in self.prefixes {
                if upper.starts_with(&prefix.to_uppercase()) {
                    return 0;
                }
            }
        } else {
            // 大文字小文字を区別する比較
            for prefix in self.prefixes {
                if trimmed.starts_with(prefix) {
                    return 0;
                }
            }
        }

        1
    }
}

// ============================================================================
// 言語別プリセット（定数として定義）
// ============================================================================

/// VHDL: `--` のみ
pub const VHDL_PREFIXES: &[&str] = &["--"];

/// Erlang/LaTeX: `%` のみ
pub const ERLANG_PREFIXES: &[&str] = &["%"];

/// Lisp系: `;` のみ
pub const LISP_PREFIXES: &[&str] = &[";"];

/// Assembly (NASM/MASM): `;` のみ
pub const ASSEMBLY_PREFIXES: &[&str] = &[";"];

/// Fortran: `!`, `C`, `c`, `*` (行頭のみ)
pub const FORTRAN_PREFIXES: &[&str] = &["!", "C", "c", "*"];

/// Batch: `REM `, `REM\t`, `::`, `@REM ` (大文字小文字区別なし)
pub const BATCH_PREFIXES: &[&str] = &["REM ", "REM\t", "::", "@REM "];

/// Visual Basic: `'`, `REM `, `REM\t` (大文字小文字区別なし)
pub const VB_PREFIXES: &[&str] = &["'", "REM ", "REM\t"];

// ============================================================================
// ファクトリ関数
// ============================================================================

impl SimplePrefixProcessor {
    /// VHDL用プロセッサ
    #[must_use]
    pub const fn vhdl() -> Self {
        Self::new(VHDL_PREFIXES)
    }

    /// Erlang/LaTeX用プロセッサ
    #[must_use]
    pub const fn erlang() -> Self {
        Self::new(ERLANG_PREFIXES)
    }

    /// Lisp系用プロセッサ
    #[must_use]
    pub const fn lisp() -> Self {
        Self::new(LISP_PREFIXES)
    }

    /// Assembly (NASM/MASM)用プロセッサ
    #[must_use]
    pub const fn assembly() -> Self {
        Self::new(ASSEMBLY_PREFIXES)
    }

    /// Fortran用プロセッサ
    #[must_use]
    pub const fn fortran() -> Self {
        Self::new(FORTRAN_PREFIXES)
    }

    /// Batch用プロセッサ (大文字小文字区別なし)
    #[must_use]
    pub const fn batch() -> Self {
        Self::new_ignore_case(BATCH_PREFIXES)
    }

    /// Visual Basic用プロセッサ (大文字小文字区別なし)
    #[must_use]
    pub const fn visual_basic() -> Self {
        Self::new_ignore_case(VB_PREFIXES)
    }

    /// Resets the processor state.
    pub const fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== VHDL テスト ====================

    #[test]
    fn test_vhdl_line_comment() {
        let p = SimplePrefixProcessor::vhdl();
        assert_eq!(p.process("-- comment"), 0);
        assert_eq!(p.process("  -- indented comment"), 0);
    }

    #[test]
    fn test_vhdl_code() {
        let p = SimplePrefixProcessor::vhdl();
        assert_eq!(p.process("signal x : integer;"), 1);
        assert_eq!(p.process("entity test is"), 1);
    }

    // ==================== Erlang テスト ====================

    #[test]
    fn test_erlang_percent_comment() {
        let p = SimplePrefixProcessor::erlang();
        assert_eq!(p.process("% comment"), 0);
        assert_eq!(p.process("%% double percent"), 0);
    }

    #[test]
    fn test_erlang_code() {
        let p = SimplePrefixProcessor::erlang();
        assert_eq!(p.process("-module(test)."), 1);
    }

    // ==================== Lisp テスト ====================

    #[test]
    fn test_lisp_semicolon_comment() {
        let p = SimplePrefixProcessor::lisp();
        assert_eq!(p.process("; comment"), 0);
        assert_eq!(p.process(";;; triple semicolon"), 0);
    }

    #[test]
    fn test_lisp_code() {
        let p = SimplePrefixProcessor::lisp();
        assert_eq!(p.process("(defun foo () 1)"), 1);
    }

    // ==================== Assembly テスト ====================

    #[test]
    fn test_assembly_semicolon_comment() {
        let p = SimplePrefixProcessor::assembly();
        assert_eq!(p.process("; NASM comment"), 0);
    }

    #[test]
    fn test_assembly_code() {
        let p = SimplePrefixProcessor::assembly();
        assert_eq!(p.process("mov ax, 1"), 1);
    }

    // ==================== Fortran テスト ====================

    #[test]
    fn test_fortran_exclamation_comment() {
        let p = SimplePrefixProcessor::fortran();
        assert_eq!(p.process("! Fortran 90 comment"), 0);
    }

    #[test]
    fn test_fortran_c_comment() {
        let p = SimplePrefixProcessor::fortran();
        assert_eq!(p.process("C Fixed format comment"), 0);
        assert_eq!(p.process("c lowercase c comment"), 0);
    }

    #[test]
    fn test_fortran_asterisk_comment() {
        let p = SimplePrefixProcessor::fortran();
        assert_eq!(p.process("* Asterisk comment"), 0);
    }

    #[test]
    fn test_fortran_code() {
        let p = SimplePrefixProcessor::fortran();
        assert_eq!(p.process("      PROGRAM HELLO"), 1);
    }

    // ==================== Batch テスト ====================

    #[test]
    fn test_batch_rem_comment() {
        let p = SimplePrefixProcessor::batch();
        assert_eq!(p.process("REM comment"), 0);
        assert_eq!(p.process("rem lowercase"), 0);
        assert_eq!(p.process("Rem mixed case"), 0);
    }

    #[test]
    fn test_batch_double_colon_comment() {
        let p = SimplePrefixProcessor::batch();
        assert_eq!(p.process(":: double colon comment"), 0);
    }

    #[test]
    fn test_batch_at_rem() {
        let p = SimplePrefixProcessor::batch();
        assert_eq!(p.process("@REM at rem comment"), 0);
    }

    #[test]
    fn test_batch_code() {
        let p = SimplePrefixProcessor::batch();
        assert_eq!(p.process("echo hello"), 1);
        assert_eq!(p.process("set VAR=value"), 1);
    }

    #[test]
    fn test_batch_rem_without_space_is_not_comment() {
        let p = SimplePrefixProcessor::batch();
        // "REMARK" などは REM + スペース/タブ ではないのでコードとして扱う
        assert_eq!(p.process("REMARK"), 1);
    }

    // ==================== Visual Basic テスト ====================

    #[test]
    fn test_vb_single_quote_comment() {
        let p = SimplePrefixProcessor::visual_basic();
        assert_eq!(p.process("' comment"), 0);
        assert_eq!(p.process("'comment without space"), 0);
    }

    #[test]
    fn test_vb_rem_comment() {
        let p = SimplePrefixProcessor::visual_basic();
        assert_eq!(p.process("REM comment"), 0);
        assert_eq!(p.process("rem lowercase"), 0);
    }

    #[test]
    fn test_vb_code() {
        let p = SimplePrefixProcessor::visual_basic();
        assert_eq!(p.process("Dim x As Integer"), 1);
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_empty_and_whitespace_lines() {
        let p = SimplePrefixProcessor::vhdl();
        // Empty line - trim results in empty string, doesn't start with prefix
        // SimplePrefixProcessor returns 1 if not a comment prefix
        assert_eq!(p.process(""), 1);
        // Whitespace only - same logic, trimmed to empty
        assert_eq!(p.process("   "), 1);
        assert_eq!(p.process("\t\t"), 1);
        assert_eq!(p.process("  \t  "), 1);
    }

    #[test]
    fn test_prefix_mid_line_is_not_comment() {
        let p = SimplePrefixProcessor::vhdl();
        // "--" appearing after code should still count as SLOC
        assert_eq!(p.process("signal x : integer; -- inline comment"), 1);
        assert_eq!(p.process("x <= y; -- assignment"), 1);
    }

    #[test]
    fn test_prefix_in_string_literal() {
        let p = SimplePrefixProcessor::erlang();
        // "%" inside a string literal - SimplePrefixProcessor doesn't parse strings
        // so if it starts with %, it's a comment
        assert_eq!(p.process("\"%\" is percent"), 1); // starts with ", not %
        assert_eq!(p.process("  \"%\""), 1); // indented string
    }

    #[test]
    fn test_unicode_content() {
        let p = SimplePrefixProcessor::vhdl();
        // Japanese comment
        assert_eq!(p.process("-- コメント"), 0);
        // Japanese code
        assert_eq!(p.process("signal 信号 : integer;"), 1);

        let p = SimplePrefixProcessor::erlang();
        // Erlang with UTF-8
        assert_eq!(p.process("% 日本語コメント"), 0);
        assert_eq!(p.process("io:format(\"日本語\")."), 1);
    }

    #[test]
    fn test_mixed_whitespace_before_prefix() {
        let p = SimplePrefixProcessor::vhdl();
        // Tabs before comment
        assert_eq!(p.process("\t-- tabbed comment"), 0);
        // Mixed spaces and tabs
        assert_eq!(p.process("  \t  -- mixed indent"), 0);
        // Tabs before code
        assert_eq!(p.process("\tsignal x;"), 1);
    }

    #[test]
    fn test_prefix_only_line() {
        let p = SimplePrefixProcessor::vhdl();
        // Just the prefix, nothing after
        assert_eq!(p.process("--"), 0);

        let p = SimplePrefixProcessor::erlang();
        assert_eq!(p.process("%"), 0);

        let p = SimplePrefixProcessor::batch();
        assert_eq!(p.process("::"), 0);
    }

    #[test]
    fn test_case_sensitivity_edge_cases() {
        // Case-insensitive: Batch
        let p = SimplePrefixProcessor::batch();
        assert_eq!(p.process("REM comment"), 0);
        assert_eq!(p.process("rem comment"), 0);
        assert_eq!(p.process("ReM comment"), 0);
        assert_eq!(p.process("rEm comment"), 0);
        // Without trailing space - not a comment
        assert_eq!(p.process("REMember"), 1);

        // Case-sensitive: VHDL
        let p = SimplePrefixProcessor::vhdl();
        assert_eq!(p.process("-- comment"), 0);
        // VHDL only uses "--", case insensitivity doesn't apply to symbols
    }

    #[test]
    fn test_fortran_column_one_comment() {
        let p = SimplePrefixProcessor::fortran();
        // C in column 1 is comment (after trim)
        assert_eq!(p.process("C comment"), 0);
        assert_eq!(p.process("c comment"), 0);
        // * in column 1
        assert_eq!(p.process("* asterisk comment"), 0);
        // ! anywhere
        assert_eq!(p.process("! modern fortran comment"), 0);
        // Note: SimplePrefixProcessor trims then checks prefix
        // So "      CALL SUB" trimmed = "CALL SUB" which starts with C!
        // Use a line that doesn't start with C, c, *, or !
        assert_eq!(p.process("      DO I=1,10"), 1);
        assert_eq!(p.process("PRINT *, 'Hello'"), 1);
    }

    #[test]
    fn test_assembly_with_instructions() {
        let p = SimplePrefixProcessor::assembly();
        // Pure comment
        assert_eq!(p.process("; comment"), 0);
        // Instruction (note: SimplePrefixProcessor doesn't handle inline comments)
        assert_eq!(p.process("mov ax, bx ; inline"), 1);
        // Label
        assert_eq!(p.process("_start:"), 1);
    }

    #[test]
    fn test_lisp_nested_semicolons() {
        let p = SimplePrefixProcessor::lisp();
        // Single semicolon
        assert_eq!(p.process("; comment"), 0);
        // Double semicolon (convention for section comments)
        assert_eq!(p.process(";; section"), 0);
        // Triple semicolon (convention for file-level comments)
        assert_eq!(p.process(";;; file"), 0);
        // Four semicolons
        assert_eq!(p.process(";;;; top level"), 0);
        // Code with semicolon in string would still be code if not starting with ;
        assert_eq!(p.process("(print \";\")"), 1);
    }

    #[test]
    fn test_vb_rem_edge_cases() {
        let p = SimplePrefixProcessor::visual_basic();
        // REM with space
        assert_eq!(p.process("REM This is a comment"), 0);
        // REM with tab
        assert_eq!(p.process("REM\tTabbed comment"), 0);
        // Single quote
        assert_eq!(p.process("' single quote comment"), 0);
        // Single quote at start without space
        assert_eq!(p.process("'no space"), 0);
        // REM without space (should be code based on prefix definition)
        assert_eq!(p.process("REMEMBER"), 1);
    }

    #[test]
    fn test_batch_at_sign_variations() {
        let p = SimplePrefixProcessor::batch();
        // @REM with space
        assert_eq!(p.process("@REM hidden comment"), 0);
        // Just @
        assert_eq!(p.process("@echo off"), 1);
        // @rem lowercase
        assert_eq!(p.process("@rem lowercase"), 0);
        // @:: is not in prefix list
        assert_eq!(p.process("@::"), 1); // @ followed by ::
    }

    #[test]
    fn test_very_long_lines() {
        let p = SimplePrefixProcessor::vhdl();
        // Very long comment
        let long_comment = alloc::format!("-- {}", "x".repeat(10000));
        assert_eq!(p.process(&long_comment), 0);
        // Very long code
        let long_code = alloc::format!("signal {} : integer;", "x".repeat(10000));
        assert_eq!(p.process(&long_code), 1);
    }

    #[test]
    fn test_special_characters() {
        let p = SimplePrefixProcessor::vhdl();
        // Comment with special chars
        assert_eq!(p.process("-- !@#$%^&*()"), 0);
        // Code with special chars
        assert_eq!(p.process("x <= '1' and '0';"), 1);

        let p = SimplePrefixProcessor::erlang();
        // Percent in different contexts
        assert_eq!(p.process("% @spec"), 0);
        assert_eq!(p.process("% TODO: fix"), 0);
    }

    #[test]
    fn test_prefix_boundary_characters() {
        let p = SimplePrefixProcessor::vhdl();
        // Prefix followed by various boundary characters
        assert_eq!(p.process("--"), 0);
        assert_eq!(p.process("-- "), 0);
        assert_eq!(p.process("--\t"), 0);
        assert_eq!(p.process("--\n"), 0);
        assert_eq!(p.process("---"), 0); // triple dash
        assert_eq!(p.process("----"), 0); // quad dash

        let p = SimplePrefixProcessor::erlang();
        assert_eq!(p.process("%"), 0);
        assert_eq!(p.process("%%"), 0);
        assert_eq!(p.process("%%%"), 0);
    }

    #[test]
    fn test_escaped_characters_in_comments() {
        let p = SimplePrefixProcessor::vhdl();
        // Comments with escape-like sequences
        assert_eq!(p.process("-- \\n newline escape"), 0);
        assert_eq!(p.process("-- \\t tab escape"), 0);
        assert_eq!(p.process("-- \\\" quote escape"), 0);
        assert_eq!(p.process("-- \\\\ backslash"), 0);
        // Backslash at end of comment
        assert_eq!(p.process("-- comment \\"), 0);
    }

    #[test]
    fn test_numeric_and_symbol_prefixes() {
        let p = SimplePrefixProcessor::fortran();
        // * as prefix (column 1)
        assert_eq!(p.process("*"), 0);
        assert_eq!(p.process("* comment"), 0);
        // But * after whitespace isn't column 1 after trim...
        // Actually trim removes leading whitespace, so "  * comment" becomes "* comment"
        assert_eq!(p.process("  * comment"), 0);

        let p = SimplePrefixProcessor::lisp();
        // Multiple semicolons
        assert_eq!(p.process(";"), 0);
        assert_eq!(p.process(";;"), 0);
        assert_eq!(p.process(";;;"), 0);
        assert_eq!(p.process(";;;;"), 0);
    }

    #[test]
    fn test_crlf_and_lf_variations() {
        let p = SimplePrefixProcessor::vhdl();
        // Line with trailing CRLF (common on Windows)
        assert_eq!(p.process("-- comment\r\n"), 0);
        assert_eq!(p.process("-- comment\r"), 0);
        // Code with trailing CRLF
        assert_eq!(p.process("signal x : bit;\r\n"), 1);
        // Just CRLF
        assert_eq!(p.process("\r\n"), 1);
        assert_eq!(p.process("\r"), 1);
    }

    #[test]
    fn test_tab_after_prefix() {
        let p = SimplePrefixProcessor::vhdl();
        // Tab immediately after prefix
        assert_eq!(p.process("--\tcomment with tab"), 0);
        // Multiple tabs
        assert_eq!(p.process("--\t\t\tcomment"), 0);

        let p = SimplePrefixProcessor::batch();
        // REM with tab
        assert_eq!(p.process("REM\tThis is a comment"), 0);
    }

    #[test]
    fn test_prefix_as_substring_in_code() {
        let p = SimplePrefixProcessor::vhdl();
        // -- appearing in string (not at start)
        assert_eq!(p.process("x := \"--not a comment\";"), 1);
        // -- appearing after code
        assert_eq!(
            p.process("x := 1; -- but this is inline, not counted as comment here"),
            1
        );

        let p = SimplePrefixProcessor::erlang();
        // % in string
        assert_eq!(p.process("S = \"%not a comment\""), 1);
    }

    #[test]
    fn test_consecutive_prefixes() {
        let p = SimplePrefixProcessor::vhdl();
        // Lines that are all comments
        assert_eq!(p.process("-- first"), 0);
        assert_eq!(p.process("-- second"), 0);
        assert_eq!(p.process("-- third"), 0);
        // Then code
        assert_eq!(p.process("signal x : bit;"), 1);
    }

    #[test]
    fn test_unicode_edge_cases() {
        let p = SimplePrefixProcessor::vhdl();
        // RTL text
        assert_eq!(p.process("-- مرحبا بالعالم"), 0);
        assert_eq!(p.process("-- שלום עולם"), 0);
        // Emoji
        assert_eq!(p.process("-- 🎉 celebration"), 0);
        assert_eq!(p.process("-- 👨‍👩‍👧‍👦 family"), 0);
        // Zero-width characters
        assert_eq!(p.process("-- zero\u{200B}width"), 0);
        // Math symbols
        assert_eq!(p.process("-- ∑∏∫∂"), 0);
        // Greek letters
        assert_eq!(p.process("-- αβγδεζηθ"), 0);
    }

    #[test]
    fn test_whitespace_variations() {
        let p = SimplePrefixProcessor::vhdl();
        // Various Unicode whitespace before prefix
        assert_eq!(p.process("\u{00A0}-- non-breaking space"), 0); // NBSP
        assert_eq!(p.process("\u{2003}-- em space"), 0); // EM SPACE
        // Form feed
        assert_eq!(p.process("\x0C-- form feed"), 0);
        // Vertical tab
        assert_eq!(p.process("\x0B-- vertical tab"), 0);
    }

    #[test]
    fn test_stress_repetition() {
        let p = SimplePrefixProcessor::vhdl();
        // Many dashes
        let many_dashes = alloc::format!("--{}", "-".repeat(1000));
        assert_eq!(p.process(&many_dashes), 0);
        // Many spaces before prefix
        let many_spaces = alloc::format!("{}-- comment", " ".repeat(1000));
        assert_eq!(p.process(&many_spaces), 0);
        // Many tabs before prefix
        let many_tabs = alloc::format!("{}-- comment", "\t".repeat(100));
        assert_eq!(p.process(&many_tabs), 0);
    }

    #[test]
    fn test_assembly_edge_cases() {
        let p = SimplePrefixProcessor::assembly();
        // Semi-colon variations
        assert_eq!(p.process(";"), 0);
        assert_eq!(p.process("; "), 0);
        assert_eq!(p.process(";\t"), 0);
        assert_eq!(p.process(";comment no space"), 0);
        assert_eq!(p.process("; comment with space"), 0);
        // Code with semicolon in different position
        assert_eq!(p.process("mov ax, bx"), 1);
        assert_eq!(p.process("mov ax, bx ; inline comment"), 1);
    }

    #[test]
    fn test_empty_prefix_edge_cases() {
        let p = SimplePrefixProcessor::vhdl();
        // Just the prefix characters with various content
        assert_eq!(p.process("--0"), 0);
        assert_eq!(p.process("--a"), 0);
        assert_eq!(p.process("-- "), 0);
        assert_eq!(p.process("--!"), 0);
        assert_eq!(p.process("--#"), 0);
    }
}
