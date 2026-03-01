// crates/core/src/language/processors/fortran_processor.rs
//! Fortran言語のコメント処理
//!
//! Fortran固有の対応:
//! - 固定形式 (Fortran 77): `C`, `c`, `*` で始まる行 (カラム1のみ)
//! - 自由形式 (Fortran 90+): `!` で始まるコメント
//!
//! # 重要
//! 固定形式のコメントは**行の1文字目**にある場合のみ有効です。
//! トリム後の判定では `count = ...` や `call ...` などの変数・関数呼び出しを
//! 誤ってコメントとして検出してしまいます。

/// Fortranプロセッサ
///
/// 固定形式と自由形式の両方のコメント構文を正しく処理します。
#[derive(Default, Clone, Debug)]
pub struct FortranProcessor;

use crate::language::processor_trait::LineProcessor;

impl LineProcessor for FortranProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        false
    }
}

impl FortranProcessor {
    /// 新しいFortranプロセッサを作成
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    ///
    /// # コメント判定ルール
    ///
    /// 1. 空行・空白のみの行 → 0
    /// 2. 固定形式コメント (カラム1に `C`, `c`, `*`) → 0
    /// 3. 自由形式コメント (トリム後 `!` で開始) → 0
    /// 4. それ以外 → 1
    ///
    /// # 例
    ///
    /// ```ignore
    /// let p = FortranProcessor::new();
    ///
    /// // 固定形式コメント (カラム1)
    /// assert_eq!(p.process("C This is a comment"), 0);
    /// assert_eq!(p.process("c lowercase"), 0);
    /// assert_eq!(p.process("* Asterisk"), 0);
    ///
    /// // 自由形式コメント
    /// assert_eq!(p.process("! Fortran 90 comment"), 0);
    /// assert_eq!(p.process("  ! Indented"), 0);
    ///
    /// // コード行 (誤判定を防ぐ)
    /// assert_eq!(p.process("      count = count + 1"), 1);
    /// assert_eq!(p.process("      call subroutine()"), 1);
    /// ```
    #[must_use]
    pub fn process(&self, line: &str) -> usize {
        // 空行・空白のみの行
        if line.trim().is_empty() {
            return 0;
        }

        // 固定形式コメント: カラム1 (行の1文字目) が C, c, * の場合
        // 注: トリムせずに生の行で判定することが重要
        if line.starts_with(['C', 'c', '*']) {
            return 0;
        }

        // 自由形式コメント: トリム後に ! で始まる行
        let trimmed = line.trim();
        if trimmed.starts_with('!') {
            return 0;
        }

        // インラインコメント (行の途中の !) は無視
        // 文字列内の ! を正しく扱うには複雑な処理が必要なため、
        // インラインコメントがあっても1行としてカウントする
        1
    }

    /// プロセッサの状態をリセット
    pub const fn reset(&mut self) {
        // Fortranプロセッサはステートレスなのでリセット不要
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 固定形式コメントテスト ====================

    #[test]
    fn test_column1_uppercase_c_comment() {
        let p = FortranProcessor::new();
        assert_eq!(p.process("C This is a fixed-form comment"), 0);
        assert_eq!(p.process("C"), 0); // C のみでもコメント
    }

    #[test]
    fn test_column1_lowercase_c_comment() {
        let p = FortranProcessor::new();
        assert_eq!(p.process("c lowercase c comment"), 0);
    }

    #[test]
    fn test_column1_asterisk_comment() {
        let p = FortranProcessor::new();
        assert_eq!(p.process("* Asterisk comment"), 0);
        assert_eq!(p.process("*"), 0);
    }

    // ==================== 自由形式コメントテスト ====================

    #[test]
    fn test_exclamation_comment_at_start() {
        let p = FortranProcessor::new();
        assert_eq!(p.process("! Fortran 90 comment"), 0);
    }

    #[test]
    fn test_exclamation_comment_indented() {
        let p = FortranProcessor::new();
        assert_eq!(p.process("  ! Indented comment"), 0);
        assert_eq!(p.process("      ! Deeply indented"), 0);
    }

    // ==================== 誤判定防止テスト (重要) ====================

    #[test]
    fn test_code_starting_with_c_not_in_column1() {
        let p = FortranProcessor::new();
        // これらは固定形式のインデントされたコード
        // カラム7以降から始まる (6文字のスペース後)
        assert_eq!(p.process("      count = count + 1"), 1);
        assert_eq!(p.process("      call subroutine()"), 1);
        assert_eq!(p.process("      character*80 str"), 1);
        assert_eq!(p.process("      complex z"), 1);
        assert_eq!(p.process("      common /block/ x, y"), 1);
    }

    #[test]
    fn test_indented_code() {
        let p = FortranProcessor::new();
        assert_eq!(p.process("      PROGRAM HELLO"), 1);
        assert_eq!(p.process("      INTEGER I, J"), 1);
        assert_eq!(p.process("      REAL X, Y"), 1);
        assert_eq!(p.process("      DO 10 I = 1, 10"), 1);
    }

    #[test]
    fn test_free_form_code() {
        let p = FortranProcessor::new();
        // 自由形式では任意のインデントが可能
        assert_eq!(p.process("program hello"), 1);
        assert_eq!(p.process("  integer :: i, j"), 1);
        assert_eq!(p.process("    do i = 1, 10"), 1);
    }

    // ==================== エッジケーステスト ====================

    #[test]
    fn test_empty_line() {
        let p = FortranProcessor::new();
        assert_eq!(p.process(""), 0);
        assert_eq!(p.process("   "), 0);
        assert_eq!(p.process("\t\t"), 0);
    }

    #[test]
    fn test_inline_comment() {
        let p = FortranProcessor::new();
        // インラインコメントがあってもコード行としてカウント
        assert_eq!(p.process("      x = 1 ! inline comment"), 1);
    }

    #[test]
    fn test_continuation_line() {
        let p = FortranProcessor::new();
        // 継続行 (カラム6に非空白文字)
        assert_eq!(p.process("     &continuation"), 1);
    }
}
