// crates/infra/src/measurement/strategies/sloc_counter/processors/sql_style.rs
//! SQL言語のコメント処理
//!
//! SQL の -- 行コメントと /* */ ブロックコメントを処理します。

use super::super::string_utils::find_outside_string_sql;

// ============================================================================
// SqlProcessor 構造体 (新設計)
// ============================================================================

/// SQL プロセッサ
///
/// - 行コメント: `--` から行末まで
/// - ブロックコメント: `/* */`
/// - 文字列リテラル (`'...'` と `"..."`) 内のコメントマーカーを無視
pub struct SqlProcessor {
    in_block_comment: bool,
}

impl SqlProcessor {
    pub fn new() -> Self {
        Self {
            in_block_comment: false,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        if self.in_block_comment {
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() {
                    return self.process(rest);
                }
            }
            return 0;
        }

        // 行コメント (文字列外)
        if let Some(line_comment_pos) = find_outside_string_sql(line, "--") {
            let before = &line[..line_comment_pos];

            // -- より前にブロックコメント開始があるかチェック
            if let Some(block_start) = find_outside_string_sql(before, "/*") {
                return self.process_block_comment(line, block_start);
            }

            return if !before.trim().is_empty() { 1 } else { 0 };
        }

        // ブロックコメント開始 (文字列外)
        if let Some(block_start) = find_outside_string_sql(line, "/*") {
            return self.process_block_comment(line, block_start);
        }

        1
    }

    fn process_block_comment(&mut self, line: &str, block_start: usize) -> usize {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();

        let after_start = &line[block_start + 2..];
        if let Some(end_offset) = after_start.find("*/") {
            let after = &after_start[end_offset + 2..];
            if has_code_before {
                return 1;
            } else if !after.trim().is_empty() {
                return self.process(after);
            }
            return 0;
        } else {
            self.in_block_comment = true;
            return if has_code_before { 1 } else { 0 };
        }
    }

    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl Default for SqlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 後方互換性のための関数 (レガシー)
// ============================================================================

/// SQL スタイル (-- と /* */) の処理
///
/// SQL の文字列リテラル ('...' と "...") 内のコメントマーカーは無視する
pub fn process_sql_style(line: &str, in_block_comment: &mut bool, count: &mut usize) {
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() {
                // 残りの部分を再帰的に処理
                process_sql_style(rest, in_block_comment, count);
            }
        }
        return;
    }

    // 行コメント (文字列外)
    if let Some(line_comment_pos) = find_outside_string_sql(line, "--") {
        // -- より前にコードがあるかチェック
        let before = &line[..line_comment_pos];

        // -- より前にブロックコメント開始があるかチェック
        if let Some(block_start) = find_outside_string_sql(before, "/*") {
            // ブロックコメントの方が先にある
            process_sql_block_comment(line, block_start, in_block_comment, count);
            return;
        }

        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }

    // ブロックコメント開始 (文字列外)
    if let Some(block_start) = find_outside_string_sql(line, "/*") {
        process_sql_block_comment(line, block_start, in_block_comment, count);
        return;
    }

    *count += 1;
}

/// SQL ブロックコメント処理のヘルパー
fn process_sql_block_comment(
    line: &str,
    block_start: usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let before = &line[..block_start];
    let has_code_before = !before.trim().is_empty();

    let after_start = &line[block_start + 2..];
    if let Some(end_offset) = after_start.find("*/") {
        // 同じ行で閉じる
        let after = &after_start[end_offset + 2..];
        if has_code_before {
            *count += 1;
        } else if !after.trim().is_empty() {
            // コメント後の残りを再帰的に処理
            process_sql_style(after, in_block_comment, count);
        }
    } else {
        // 閉じられていない = ブロックコメント開始
        *in_block_comment = true;
        if has_code_before {
            *count += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style("-- comment", &mut in_block, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_sql_code_with_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style("SELECT * FROM t; -- comment", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_string_with_block_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style("SELECT '/* これはコメントではありません */' FROM users;", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_string_with_line_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style("SELECT '-- これもコメントではない' FROM users;", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_escaped_quote() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style("SELECT 'It''s a test /* not comment */' FROM t;", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_double_quote_identifier() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style(r#"SELECT "column /* name */" FROM t;"#, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_real_block_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_sql_style("SELECT * /* comment */ FROM t;", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_sql_block_comment_multiline() {
        let mut in_block = false;
        let mut count = 0;

        process_sql_style("/*", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_sql_style("  comment", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_sql_style("*/", &mut in_block, &mut count);
        assert!(!in_block);
        assert_eq!(count, 0);

        process_sql_style("SELECT 1;", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }
}
