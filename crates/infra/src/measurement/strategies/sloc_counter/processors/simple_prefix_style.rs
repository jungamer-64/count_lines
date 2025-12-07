// crates/infra/src/measurement/strategies/sloc_counter/processors/simple_prefix_style.rs
//! 単純なプレフィックス型コメントプロセッサ
//!
//! 「特定のプレフィックスで始まる行をコメントとする」言語群を統合。
//! 対象: Batch, VHDL, Erlang, Lisp, Fortran, Assembly など

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
pub struct SimplePrefixProcessor {
    prefixes: &'static [&'static str],
    ignore_case: bool,
}

impl SimplePrefixProcessor {
    /// 大文字小文字を区別するプロセッサを作成
    pub const fn new(prefixes: &'static [&'static str]) -> Self {
        Self {
            prefixes,
            ignore_case: false,
        }
    }

    /// 大文字小文字を区別しないプロセッサを作成
    pub const fn new_ignore_case(prefixes: &'static [&'static str]) -> Self {
        Self {
            prefixes,
            ignore_case: true,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
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
    pub fn vhdl() -> Self {
        Self::new(VHDL_PREFIXES)
    }

    /// Erlang/LaTeX用プロセッサ
    pub fn erlang() -> Self {
        Self::new(ERLANG_PREFIXES)
    }

    /// Lisp系用プロセッサ
    pub fn lisp() -> Self {
        Self::new(LISP_PREFIXES)
    }

    /// Assembly (NASM/MASM)用プロセッサ
    pub fn assembly() -> Self {
        Self::new(ASSEMBLY_PREFIXES)
    }

    /// Fortran用プロセッサ
    pub fn fortran() -> Self {
        Self::new(FORTRAN_PREFIXES)
    }

    /// Batch用プロセッサ (大文字小文字区別なし)
    pub fn batch() -> Self {
        Self::new_ignore_case(BATCH_PREFIXES)
    }

    /// Visual Basic用プロセッサ (大文字小文字区別なし)
    pub fn visual_basic() -> Self {
        Self::new_ignore_case(VB_PREFIXES)
    }
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
}
