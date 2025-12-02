// crates/infra/src/measurement/strategies/sloc_counter/processors/dlang_style.rs
//! D言語のコメント処理
//!
//! D言語は3種類のコメントをサポート:
//! - `//` 行コメント
//! - `/* */` ブロックコメント（ネスト不可）
//! - `/+ +/` ブロックコメント（ネスト可能）

use super::super::string_utils::find_outside_string;

/// D言語スタイル処理 (//, /* */, /+ +/ ネスト対応)
///
/// D言語は3種類のコメントをサポート:
/// - `//` 行コメント
/// - `/* */` ブロックコメント（ネスト不可）
/// - `/+ +/` ブロックコメント（ネスト可能）
pub fn process_dlang_style(
    line: &str,
    in_block_comment: &mut bool,
    in_nesting_block: &mut bool,
    nesting_block_depth: &mut usize,
    count: &mut usize,
) {
    // /+ +/ ネストブロックコメント内
    if *nesting_block_depth > 0 {
        process_dlang_nesting_block(line, in_nesting_block, nesting_block_depth, in_block_comment, count);
        return;
    }
    
    // 通常の /* */ ブロックコメント内
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                // 残りを再帰処理
                process_dlang_style(rest, in_block_comment, in_nesting_block, nesting_block_depth, count);
            }
        }
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string(line, "//") {
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            return;
        }
        *count += 1;
        return;
    }

    // /+ ネストブロックコメント開始をチェック（文字列外）
    if let Some(nesting_start) = find_outside_string(line, "/+") {
        // /* より前に /+ があるかチェック
        let block_start = find_outside_string(line, "/*");
        
        if block_start.is_none() || nesting_start < block_start.unwrap() {
            // /+ が先
            let before = &line[..nesting_start];
            let has_code_before = !before.trim().is_empty();
            
            *nesting_block_depth = 1;
            *in_nesting_block = true;
            let rest = &line[nesting_start + 2..];
            process_dlang_nesting_block(rest, in_nesting_block, nesting_block_depth, in_block_comment, count);
            
            if has_code_before {
                *count += 1;
            }
            return;
        }
    }

    // /* ブロックコメント開始をチェック（文字列外）
    if let Some(block_start) = find_outside_string(line, "/*") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        if let Some(block_end) = line[block_start + 2..].find("*/") {
            let after = &line[block_start + 2 + block_end + 2..];
            if has_code_before {
                *count += 1;
            } else if !after.trim().is_empty() {
                // 残りを再帰処理
                process_dlang_style(after, in_block_comment, in_nesting_block, nesting_block_depth, count);
            }
        } else {
            *in_block_comment = true;
            if has_code_before {
                *count += 1;
            }
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// D言語の /+ +/ ネストブロックコメント行を処理
fn process_dlang_nesting_block(
    line: &str,
    in_nesting_block: &mut bool,
    nesting_block_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        if i + 1 < bytes.len() {
            // /+ を見つけたらネスト深度を増やす
            if bytes[i] == b'/' && bytes[i + 1] == b'+' {
                *nesting_block_depth += 1;
                i += 2;
                continue;
            }
            // +/ を見つけたらネスト深度を減らす
            if bytes[i] == b'+' && bytes[i + 1] == b'/' {
                *nesting_block_depth -= 1;
                i += 2;
                
                // 全てのコメントが閉じた
                if *nesting_block_depth == 0 {
                    *in_nesting_block = false;
                    let rest = &line[i..];
                    if !rest.trim().is_empty() {
                        // 残りの部分を再帰的に処理
                        process_dlang_style(rest, in_block_comment, in_nesting_block, nesting_block_depth, count);
                    }
                    return;
                }
                continue;
            }
        }
        i += 1;
    }
    
    // まだコメント内
    *in_nesting_block = *nesting_block_depth > 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== D 言語 行コメントテスト ====================

    #[test]
    fn test_dlang_line_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("// comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int y = 2; // inline comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_dlang_line_comment_only() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("// This is a comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_dlang_code_with_inline_line_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("int x = 42; // answer", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== D 言語 ブロックコメントテスト ====================

    #[test]
    fn test_dlang_block_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/* block comment */", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int z = 3;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_block_comment_multiline() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/* start of", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert!(in_block);
        process_dlang_style("   multiline", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("   comment */", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert!(!in_block);
        process_dlang_style("int a = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_code_before_block_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("int x = 1; /* comment */", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_block_comment_with_code_after() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/* comment */ int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== D 言語 /+ +/ ネストコメントテスト ====================

    #[test]
    fn test_dlang_nesting_comment_basic() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("  nesting comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int a = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_nested() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ outer", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ inner +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("still in outer +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int b = 2;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_single_line() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ /+ nested +/ still in outer +/ int c = 3;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_deep() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ level 1", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ level 2", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ level 3 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("back to level 2 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("back to level 1 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int d = 4;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_very_deep() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ 1", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ 2", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ 3", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ 4 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int e = 5;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== D 言語 混合テスト ====================

    #[test]
    fn test_dlang_mixed_comments() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/* block */ /+ nesting +/ int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_code_before_nesting_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("int x = 1; /+ comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ nested +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int y = 2;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_dlang_nesting_before_block() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        // /+ が /* より先に現れる場合
        process_dlang_style("/+ nesting +/ /* block */", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_dlang_block_before_nesting() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        // /* が /+ より先に現れる場合（この行では /* がないので /+ が処理される）
        process_dlang_style("int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== D 言語 エッジケーステスト ====================

    #[test]
    fn test_dlang_empty_nesting_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/++/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_with_code_after_on_same_line() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ comment +/ int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_consecutive_comments() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("// line 1", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("// line 2", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/* block */", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ nesting +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }
}
