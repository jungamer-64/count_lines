// crates/infra/src/measurement/strategies/sloc_counter/processors/haskell_style.rs
//! Haskell言語のコメント処理
//!
//! Haskell固有の対応:
//! - 行コメント: `--`
//! - ブロックコメント: `{-` ～ `-}` (ネスト対応)

// ============================================================================
// HaskellProcessor 構造体 (新設計)
// ============================================================================

/// Haskell プロセッサ
///
/// - 行コメント: `--`
/// - ブロックコメント: `{- -}` (ネスト対応)
pub struct HaskellProcessor {
    block_comment_depth: usize,
}

impl HaskellProcessor {
    pub fn new() -> Self {
        Self {
            block_comment_depth: 0,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        // ネストされたブロックコメント内
        if self.block_comment_depth > 0 {
            self.process_nesting_block(line);
            return 0;
        }

        // 行コメント
        if line.starts_with("--") {
            return 0;
        }

        // ブロックコメント開始 {-
        if let Some(block_start) = line.find("{-") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

            self.block_comment_depth = 1;
            let rest = &line[block_start + 2..];
            let rest_has_code = self.process_nesting_block(rest);

            return if has_code_before || rest_has_code { 1 } else { 0 };
        }

        1
    }

    /// ネストされたブロックコメント行を処理、返り値はコメント終了後にコードがあるかどうか
    fn process_nesting_block(&mut self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if i + 1 < bytes.len() {
                // {- を見つけたらネスト深度を増やす
                if bytes[i] == b'{' && bytes[i + 1] == b'-' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                // -} を見つけたらネスト深度を減らす
                if bytes[i] == b'-' && bytes[i + 1] == b'}' {
                    self.block_comment_depth -= 1;
                    i += 2;

                    if self.block_comment_depth == 0 {
                        let rest = &line[i..];
                        if !rest.trim().is_empty() {
                            return self.process(rest) > 0;
                        }
                        return false;
                    }
                    continue;
                }
            }
            i += 1;
        }

        false
    }

    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.block_comment_depth > 0
    }
}

impl Default for HaskellProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 後方互換性のための関数 (レガシー)
// ============================================================================

/// Haskell スタイル (-- と {- -}) の処理 - ネスト対応
///
/// Haskell のブロックコメント `{- -}` はネスト可能
pub fn process_haskell_style(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
    count: &mut usize,
) {
    // ネストされたブロックコメント内
    if *block_comment_depth > 0 {
        process_nesting_haskell_block(line, block_comment_depth, in_block_comment, count);
        return;
    }

    // 行コメント
    if line.starts_with("--") {
        return;
    }

    // ブロックコメント開始 {-
    if let Some(block_start) = line.find("{-") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

        // ブロックコメント開始
        *block_comment_depth = 1;
        let rest = &line[block_start + 2..];
        process_nesting_haskell_block(rest, block_comment_depth, in_block_comment, count);

        if has_code_before {
            *count += 1;
        }
        return;
    }

    *count += 1;
}

/// ネストされた Haskell ブロックコメント行を処理
fn process_nesting_haskell_block(
    line: &str,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() {
            // {- を見つけたらネスト深度を増やす
            if bytes[i] == b'{' && bytes[i + 1] == b'-' {
                *block_comment_depth += 1;
                i += 2;
                continue;
            }
            // -} を見つけたらネスト深度を減らす
            if bytes[i] == b'-' && bytes[i + 1] == b'}' {
                *block_comment_depth -= 1;
                i += 2;

                // 全てのコメントが閉じた
                if *block_comment_depth == 0 {
                    let rest = &line[i..];
                    if !rest.trim().is_empty() {
                        // 残りの部分を再帰的に処理
                        process_haskell_style(rest, in_block_comment, block_comment_depth, count);
                    }
                    return;
                }
                continue;
            }
        }
        i += 1;
    }

    // in_block_comment フラグも同期
    *in_block_comment = *block_comment_depth > 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    // テストヘルパー: 複数行を処理
    fn process_lines(lines: &[&str]) -> usize {
        let mut count = 0;
        let mut in_block = false;
        let mut depth = 0;
        for line in lines {
            process_haskell_style(line, &mut in_block, &mut depth, &mut count);
        }
        count
    }

    #[test]
    fn test_line_comment() {
        let count = process_lines(&[
            "-- comment",
            "x = 1",
        ]);
        // x = 1 は行コメントの後なのでSLOC
        assert_eq!(count, 1);
    }

    #[test]
    fn test_nested_comment() {
        let count = process_lines(&[
            "{-",
            "  Outer comment",
            "  {- Inner comment -}",
            "  Still outer comment",
            "-}",
            "main = putStrLn \"Hello\"",
        ]);
        // main の1行がSLOC
        assert_eq!(count, 1);
    }

    #[test]
    fn test_nested_comment_deep() {
        let count = process_lines(&[
            "{- level 1",
            "{- level 2",
            "{- level 3 -}",
            "back to level 2 -}",
            "back to level 1 -}",
            "x = 1",
        ]);
        // x の1行がSLOC
        assert_eq!(count, 1);
    }

    #[test]
    fn test_nested_comment_single_line() {
        let count = process_lines(&[
            "{- {- nested -} still comment -} x = 1",
        ]);
        // x = 1 の1行がSLOC
        assert_eq!(count, 1);
    }
}
