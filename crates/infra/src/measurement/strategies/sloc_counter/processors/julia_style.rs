// crates/infra/src/measurement/strategies/sloc_counter/processors/julia_style.rs
//! Julia言語のコメント処理
//!
//! Julia のコメント構文:
//! - 行コメント: `#`
//! - ブロックコメント: `#= ... =#` (ネスト対応)

use super::super::string_utils::find_outside_string;

/// Julia スタイル (# と #= =#) の処理
/// 
/// # Arguments
/// * `line` - 処理する行
/// * `in_block_comment` - ブロックコメント内かどうか
/// * `block_comment_depth` - ブロックコメントのネスト深度
/// * `count` - SLOCカウント
pub fn process_julia_style(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
    count: &mut usize,
) {
    let trimmed = line.trim();
    
    // ブロックコメント内の処理
    if *in_block_comment {
        let remaining = process_julia_block_comment(line, in_block_comment, block_comment_depth);
        // ブロックコメント終了後にコードがあればカウント
        if !*in_block_comment {
            if let Some(rest) = remaining {
                if !rest.trim().is_empty() {
                    // 残りの部分を再帰的に処理
                    process_julia_style(rest, in_block_comment, block_comment_depth, count);
                }
            }
        }
        return;
    }
    
    // 空行
    if trimmed.is_empty() {
        return;
    }
    
    // ブロックコメント開始判定
    if let Some(pos) = find_outside_string(line, "#=") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        // ブロックコメント開始
        *in_block_comment = true;
        *block_comment_depth = 1;
        
        // 残りの部分でさらにネストや終了をチェック
        let rest = &line[pos + 2..];
        let remaining = check_julia_block_nesting(rest, in_block_comment, block_comment_depth);
        
        // ブロックコメント終了後にコードがあればカウント
        if !*in_block_comment {
            if let Some(rest_code) = remaining {
                if !rest_code.trim().is_empty() {
                    // 残りの部分を再帰的に処理
                    process_julia_style(rest_code, in_block_comment, block_comment_depth, count);
                }
            }
        }
        return;
    }
    
    // 行コメント判定
    if let Some(hash_pos) = find_outside_string(line, "#") {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }
    
    // コードとしてカウント
    *count += 1;
}

/// ブロックコメント内の処理（ネスト対応）
/// コメント終了後の残りの文字列を返す
fn process_julia_block_comment<'a>(
    line: &'a str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
) -> Option<&'a str> {
    check_julia_block_nesting(line, in_block_comment, block_comment_depth)
}

/// Julia ブロックコメントのネスト処理
/// コメント終了後の残りの文字列を返す
fn check_julia_block_nesting<'a>(
    content: &'a str,
    in_block_comment: &mut bool,
    depth: &mut usize,
) -> Option<&'a str> {
    let bytes = content.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // ネストされた #= の開始
        if i + 1 < bytes.len() && bytes[i] == b'#' && bytes[i + 1] == b'=' {
            *depth += 1;
            i += 2;
            continue;
        }
        
        // =# の終了
        if i + 1 < bytes.len() && bytes[i] == b'=' && bytes[i + 1] == b'#' {
            *depth = depth.saturating_sub(1);
            if *depth == 0 {
                *in_block_comment = false;
                // コメント終了後の残りを返す
                return Some(&content[i + 2..]);
            }
            i += 2;
            continue;
        }
        
        i += 1;
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julia_line_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_julia_style("# comment", &mut in_block, &mut depth, &mut count);
        process_julia_style("x = 1", &mut in_block, &mut depth, &mut count);
        process_julia_style("y = 2 # inline comment", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_julia_block_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_julia_style("#=", &mut in_block, &mut depth, &mut count);
        assert!(in_block);
        process_julia_style("  block comment", &mut in_block, &mut depth, &mut count);
        process_julia_style("=#", &mut in_block, &mut depth, &mut count);
        assert!(!in_block);
        process_julia_style("z = 3", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_julia_nested_block_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_julia_style("#= outer", &mut in_block, &mut depth, &mut count);
        process_julia_style("#= inner =#", &mut in_block, &mut depth, &mut count);
        process_julia_style("still in outer =#", &mut in_block, &mut depth, &mut count);
        process_julia_style("a = 1", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
        assert!(!in_block);
    }

    #[test]
    fn test_julia_block_comment_single_line() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_julia_style("#= comment =# b = 2", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
        assert!(!in_block);
    }
}
