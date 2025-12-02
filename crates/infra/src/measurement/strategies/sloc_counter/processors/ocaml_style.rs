// crates/infra/src/measurement/strategies/sloc_counter/processors/ocaml_style.rs
//! OCaml/F#/Pascal系言語のコメント処理
//!
//! 対応するコメント構文:
//! - ブロックコメント: `(* ... *)` (ネスト対応)
//! - F# は // 行コメントも持つが、ここでは (* *) のみを処理

use super::super::string_utils::find_outside_string;

/// OCaml スタイル ((* *)) の処理
/// 
/// # Arguments
/// * `line` - 処理する行
/// * `in_block_comment` - ブロックコメント内かどうか
/// * `block_comment_depth` - ブロックコメントのネスト深度
/// * `count` - SLOCカウント
pub fn process_ocaml_style(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
    count: &mut usize,
) {
    let trimmed = line.trim();
    
    // ブロックコメント内の処理
    if *in_block_comment {
        process_ocaml_block_comment(line, in_block_comment, block_comment_depth);
        return;
    }
    
    // 空行
    if trimmed.is_empty() {
        return;
    }
    
    // F#/OCaml の // 行コメント対応（オプション）
    // 注: OCaml は // をサポートしないが、F# はサポートする
    // ここでは両方に対応するため // もチェック
    if let Some(pos) = find_outside_string(line, "//") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }
    
    // ブロックコメント開始判定
    if let Some(pos) = find_outside_string(line, "(*") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        // ブロックコメント開始
        *in_block_comment = true;
        *block_comment_depth = 1;
        
        // 残りの部分でさらにネストや終了をチェック
        let rest = &line[pos + 2..];
        check_ocaml_block_nesting(rest, in_block_comment, block_comment_depth);
        return;
    }
    
    // コードとしてカウント
    *count += 1;
}

/// ブロックコメント内の処理（ネスト対応）
fn process_ocaml_block_comment(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
) {
    check_ocaml_block_nesting(line, in_block_comment, block_comment_depth);
}

/// OCaml ブロックコメントのネスト処理
fn check_ocaml_block_nesting(
    content: &str,
    in_block_comment: &mut bool,
    depth: &mut usize,
) {
    let bytes = content.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // ネストされた (* の開始
        if i + 1 < bytes.len() && bytes[i] == b'(' && bytes[i + 1] == b'*' {
            *depth += 1;
            i += 2;
            continue;
        }
        
        // *) の終了
        if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b')' {
            *depth = depth.saturating_sub(1);
            if *depth == 0 {
                *in_block_comment = false;
            }
            i += 2;
            continue;
        }
        
        i += 1;
    }
}
