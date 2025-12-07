// crates/infra/src/measurement/strategies/sloc_counter/processors/swift_style.rs
//! Swift言語のコメント処理
//!
//! Swift固有の対応:
//! - `//` 行コメント
//! - `/* */` ブロックコメント（ネスト対応）
//! - 拡張デリミタ文字列 `#"..."#`, `##"..."##` 等
//! - 多重引用符文字列 `"""..."""`

use super::super::string_utils::find_outside_string_swift;

// ============================================================================
// SwiftProcessor 構造体 (新設計)
// ============================================================================

/// Swift プロセッサ
///
/// Swift特有の機能を処理:
/// - ネストされたブロックコメント (`/* /* */ */`)
/// - 拡張デリミタ文字列 (`#"..."#`)
/// - 多重引用符文字列 (`"""..."""`)
pub struct SwiftProcessor {
    block_comment_depth: usize,
    in_block_comment: bool,
}

impl SwiftProcessor {
    pub fn new() -> Self {
        Self {
            block_comment_depth: 0,
            in_block_comment: false,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        // ネストされたブロックコメント内
        if self.block_comment_depth > 0 {
            self.process_nesting_block_line(line);
            return 0;
        }

        // 行コメント（文字列外）のみの行かチェック
        if let Some(line_comment_pos) = find_outside_string_swift(line, "//") {
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        // ブロックコメント開始をチェック（文字列外）
        if let Some(block_start) = find_outside_string_swift(line, "/*") {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();

            // ブロックコメント開始後の部分を処理
            self.block_comment_depth = 1;
            self.in_block_comment = true;
            let rest = &line[block_start + 2..];
            
            // 残りの部分を処理（同一行での閉じがあるか確認）
            let rest_has_code = self.process_nesting_block_line(rest);

            if has_code_before || rest_has_code {
                return 1;
            }
            return 0;
        }

        // コードがある行
        1
    }

    /// ネストされたブロックコメント行を処理
    /// 戻り値: コメント終了後にコードがあるかどうか
    fn process_nesting_block_line(&mut self, line: &str) -> bool {
        let bytes = line.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if i + 1 < bytes.len() {
                // /* を見つけたらネスト深度を増やす
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                // */ を見つけたらネスト深度を減らす
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    self.block_comment_depth -= 1;
                    i += 2;

                    // 全てのコメントが閉じた
                    if self.block_comment_depth == 0 {
                        self.in_block_comment = false;
                        let rest = &line[i..];
                        if !rest.trim().is_empty() {
                            // 残りの部分を再帰的に処理
                            return self.process(rest) > 0;
                        }
                        return false;
                    }
                    continue;
                }
            }
            i += 1;
        }

        // in_block_comment フラグも同期
        self.in_block_comment = self.block_comment_depth > 0;
        false
    }

    /// ブロックコメント内かどうか（テスト用）
    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment || self.block_comment_depth > 0
    }
}

impl Default for SwiftProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 後方互換性のための関数 (レガシー)
// ============================================================================

/// Swift スタイル処理（拡張デリミタ文字列 #"..."# と多重引用符 """...""" 対応）
pub fn process_swift_style(
    line: &str,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    // ネストされたブロックコメント内
    if *block_comment_depth > 0 {
        process_nesting_block_comment_line(line, block_comment_depth, in_block_comment, count);
        return;
    }

    // 行コメント（文字列外）のみの行かチェック - Swift文字列対応
    if let Some(line_comment_pos) = find_outside_string_swift(line, "//") {
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            return;
        }
        *count += 1;
        return;
    }

    // ブロックコメント開始をチェック（文字列外、Swift文字列対応）
    if let Some(block_start) = find_outside_string_swift(line, "/*") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメント開始後の部分を処理
        *block_comment_depth = 1;
        let rest = &line[block_start + 2..];
        process_nesting_block_comment_line(rest, block_comment_depth, in_block_comment, count);
        
        if has_code_before {
            *count += 1;
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// ネストされたブロックコメント行を処理
fn process_nesting_block_comment_line(
    line: &str,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        if i + 1 < bytes.len() {
            // /* を見つけたらネスト深度を増やす
            if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                *block_comment_depth += 1;
                i += 2;
                continue;
            }
            // */ を見つけたらネスト深度を減らす
            if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                *block_comment_depth -= 1;
                i += 2;
                
                // 全てのコメントが閉じた
                if *block_comment_depth == 0 {
                    let rest = &line[i..];
                    if !rest.trim().is_empty() {
                        // 残りの部分を再帰的に処理
                        process_swift_style(
                            rest,
                            block_comment_depth,
                            in_block_comment,
                            count,
                        );
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

    // ==================== SwiftProcessor テスト ====================

    #[test]
    fn test_swift_processor_line_comment() {
        let mut p = SwiftProcessor::new();
        assert_eq!(p.process("// comment"), 0);
        assert_eq!(p.process("let x = 1"), 1);
    }

    #[test]
    fn test_swift_processor_nested_block_comment() {
        let mut p = SwiftProcessor::new();
        assert_eq!(p.process("/* outer"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("/* nested */"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("let x = 1"), 1);
    }

    #[test]
    fn test_swift_processor_extended_delimiter() {
        let mut p = SwiftProcessor::new();
        assert_eq!(p.process(r##"let s = #"/* not a comment */"#"##), 1);
    }

    // ==================== 後方互換関数テスト ====================
    // ==================== Swift ネストコメントテスト ====================

    #[test]
    fn test_swift_nested_comments() {

        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("/* outer", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("  /* nested */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("*/", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_swift_nested_comments_single_line() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("/* /* nested */ still comment */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_swift_nested_comments_deep() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("/* level 1", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("/* level 2", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("/* level 3 */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("back to level 2 */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("back to level 1 */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let y = 2", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== Swift 拡張デリミタ文字列テスト ====================

    #[test]
    fn test_swift_extended_delimiter_string() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r##"let s = #"/* not a comment */"#"##, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_swift_extended_delimiter_double_hash() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r###"let s = ##"contains "# but not end"##"###, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let y = 2", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_swift_extended_delimiter_with_line_comment() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r##"let s = #"// not a comment"#"##, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let z = 3", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Swift 多重引用符文字列テスト ====================

    #[test]
    fn test_swift_multiline_string() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r#"let s = """/* not a comment */""""#, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let z = 3", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_swift_multiline_string_with_line_comment_marker() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r#"let s = """// not a comment""""#, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let w = 4", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Swift 行コメントテスト ====================

    #[test]
    fn test_swift_line_comment() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("// Swift comment", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_swift_code_with_inline_comment() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("let x = 1 // comment", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== Swift その他のテスト ====================

    #[test]
    fn test_swift_hash_not_comment() {
        // Swift では # はコメント開始ではない（拡張デリミタの一部）
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("let hash = #selector(foo)", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_swift_code_before_block_comment() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("let x = 1 /* comment", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("/* nested */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("*/", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let y = 2", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_swift_block_comment_with_code_after() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("/* comment */ let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_swift_empty_line() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1); // 空行もコードとしてカウント（trim後判定は呼び出し元で実施）
    }
}
