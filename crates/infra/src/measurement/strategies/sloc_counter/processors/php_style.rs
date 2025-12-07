// crates/infra/src/measurement/strategies/sloc_counter/processors/php_style.rs
//! PHP のコメント処理
//!
//! PHP は C系の `//, /* */` に加えて、Perl/Shell系の `#` 行コメントもサポートします。

use super::super::string_utils::find_outside_string;

// ============================================================================
// PhpProcessor 構造体 (新設計)
// ============================================================================

/// PHP プロセッサ
///
/// PHP は以下のコメント形式をサポート:
/// - 行コメント: `//` から行末
/// - 行コメント: `#` から行末
/// - ブロックコメント: `/* */`
pub struct PhpProcessor {
    in_block_comment: bool,
}

impl PhpProcessor {
    pub fn new() -> Self {
        Self {
            in_block_comment: false,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        if self.in_block_comment {
            // ブロックコメント内
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                // 閉じた後にコードがあるかチェック
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty()
                    && !rest.trim().starts_with("//")
                    && !rest.trim().starts_with('#')
                {
                    return 1;
                }
            }
            return 0;
        }

        // 行コメント // （文字列外）
        let line_comment_pos = find_outside_string(line, "//");

        // 行コメント # （文字列外）
        let hash_comment_pos = find_outside_string(line, "#");

        // ブロックコメント開始 /* （文字列外）
        let block_start_pos = find_outside_string(line, "/*");

        // 最初に出現するコメントマーカーを特定
        let first_comment = [line_comment_pos, hash_comment_pos, block_start_pos]
            .into_iter()
            .flatten()
            .min();

        match first_comment {
            None => {
                // コメントなし = コード行
                1
            }
            Some(pos) => {
                let before = &line[..pos];
                let has_code_before = !before.trim().is_empty();

                // ブロックコメント開始が最初か判定
                if block_start_pos == Some(pos) {
                    // ブロックコメント
                    if let Some(end_offset) = line[pos + 2..].find("*/") {
                        // 同じ行で閉じる
                        let after = &line[pos + 2 + end_offset + 2..];
                        let has_code_after = !after.trim().is_empty()
                            && !after.trim().starts_with("//")
                            && !after.trim().starts_with('#');
                        if has_code_before || has_code_after {
                            return 1;
                        }
                    } else {
                        // 次の行に続く
                        self.in_block_comment = true;
                        if has_code_before {
                            return 1;
                        }
                    }
                    0
                } else {
                    // 行コメント (// または #)
                    if has_code_before { 1 } else { 0 }
                }
            }
        }
    }

    /// ブロックコメント内かどうか（テスト用）
    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl Default for PhpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 後方互換性のための関数 (レガシー)
// ============================================================================

/// PHP スタイル (// と /* */ と #) の処理
///
/// PHPは以下のコメント形式をサポート:
/// - 行コメント: `//` から行末
/// - 行コメント: `#` から行末
/// - ブロックコメント: `/* */`
pub fn process_php_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        // ブロックコメント内
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            // 閉じた後にコードがあるかチェック
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty()
                && !rest.trim().starts_with("//")
                && !rest.trim().starts_with('#')
            {
                *count += 1;
            }
        }
        return;
    }

    // 行コメント // （文字列外）
    let line_comment_pos = find_outside_string(line, "//");

    // 行コメント # （文字列外）
    let hash_comment_pos = find_outside_string(line, "#");

    // ブロックコメント開始 /* （文字列外）
    let block_start_pos = find_outside_string(line, "/*");

    // 最初に出現するコメントマーカーを特定
    let first_comment = [line_comment_pos, hash_comment_pos, block_start_pos]
        .into_iter()
        .flatten()
        .min();

    match first_comment {
        None => {
            // コメントなし = コード行
            *count += 1;
        }
        Some(pos) => {
            let before = &line[..pos];
            let has_code_before = !before.trim().is_empty();

            // ブロックコメント開始が最初か判定
            if block_start_pos == Some(pos) {
                // ブロックコメント
                if let Some(end_offset) = line[pos + 2..].find("*/") {
                    // 同じ行で閉じる
                    let after = &line[pos + 2 + end_offset + 2..];
                    let has_code_after = !after.trim().is_empty()
                        && !after.trim().starts_with("//")
                        && !after.trim().starts_with('#');
                    if has_code_before || has_code_after {
                        *count += 1;
                    }
                } else {
                    // 次の行に続く
                    *in_block_comment = true;
                    if has_code_before {
                        *count += 1;
                    }
                }
            } else {
                // 行コメント (// または #)
                if has_code_before {
                    *count += 1;
                }
                // コメントのみの行は何もしない
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== PhpProcessor テスト ====================

    #[test]
    fn test_php_processor_line_comment() {
        let mut p = PhpProcessor::new();
        assert_eq!(p.process("// comment"), 0);
        assert_eq!(p.process("# comment"), 0);
    }

    #[test]
    fn test_php_processor_code() {
        let mut p = PhpProcessor::new();
        assert_eq!(p.process("$x = 1;"), 1);
    }

    #[test]
    fn test_php_processor_block_comment() {
        let mut p = PhpProcessor::new();
        assert_eq!(p.process("/* start"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("middle"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("echo 1;"), 1);
    }

    // ==================== 後方互換関数テスト ====================

    #[test]
    fn test_php_line_comment_double_slash() {
        let mut in_block = false;
        let mut count = 0;
        process_php_style("// comment", &mut in_block, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_php_line_comment_hash() {
        let mut in_block = false;
        let mut count = 0;
        process_php_style("# comment", &mut in_block, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_php_code_with_hash_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_php_style("$x = 1; # comment", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_php_code_with_double_slash_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_php_style("$y = 2; // comment", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_php_block_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_php_style("/* block comment */", &mut in_block, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_php_block_comment_multiline() {
        let mut in_block = false;
        let mut count = 0;

        process_php_style("/* start", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_php_style("middle", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_php_style("*/", &mut in_block, &mut count);
        assert!(!in_block);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_php_hash_in_string() {
        let mut in_block = false;
        let mut count = 0;
        // 文字列内の # はコメントではない
        process_php_style(r#"$s = "Hello # World";"#, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_php_hash_before_double_slash() {
        let mut in_block = false;
        let mut count = 0;
        // # が // より先に出現
        process_php_style("$x = 1; # comment // not reached", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }
}
