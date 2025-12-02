// crates/infra/src/measurement/strategies/sloc_counter/processors/c_style.rs
//! C系言語のコメント処理
//!
//! C/C++/Java/JavaScript/Rust/Go/Swift/Kotlin等の
//! `//` 行コメントと `/* */` ブロックコメントを処理します。

use super::super::string_utils::{
    find_outside_string, find_outside_string_swift,
    find_outside_string_with_options, StringSkipOptions,
};

/// C系スタイル (// と /* */) の処理 - StringSkipOptions対応版
/// 
/// 言語に応じたStringSkipOptionsを渡すことで、
/// 各言語固有の文字列リテラル構文を正しくスキップできます。
pub fn process_c_style_with_options(
    line: &str,
    options: &StringSkipOptions,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        // ブロックコメント内
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            // 閉じた後にコードがあるかチェック
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                *count += 1;
            }
        }
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", options) {
        // // より前にコードがあるか
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            // コメントのみの行
            return;
        }
        // コメント前にコードがある
        *count += 1;
        return;
    }

    // ブロックコメント開始をチェック（文字列外）
    if let Some(block_start) = find_outside_string_with_options(line, "/*", options) {
        // /* より前にコードがあるか
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメントが同じ行で閉じるか
        if let Some(block_end) = line[block_start + 2..].find("*/") {
            let after = &line[block_start + 2 + block_end + 2..];
            let has_code_after = !after.trim().is_empty() 
                && find_outside_string_with_options(after, "//", options).is_none_or(|p| p > 0);
            if has_code_before || has_code_after {
                *count += 1;
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

/// ネストコメント対応 C系スタイル処理 - StringSkipOptions対応版
///
/// 言語に応じたStringSkipOptionsを渡すことで、
/// 各言語固有の文字列リテラル構文を正しくスキップできます。
pub fn process_nesting_c_style_with_options(
    line: &str,
    options: &StringSkipOptions,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    // ネストされたブロックコメント内
    if *block_comment_depth > 0 {
        process_nesting_block_comment_line_with_options(
            line,
            options,
            block_comment_depth,
            in_block_comment,
            count,
        );
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", options) {
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            return;
        }
        *count += 1;
        return;
    }

    // ブロックコメント開始をチェック（文字列外）
    if let Some(block_start) = find_outside_string_with_options(line, "/*", options) {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメント開始後の部分を処理
        *block_comment_depth = 1;
        let rest = &line[block_start + 2..];
        process_nesting_block_comment_line_with_options(
            rest,
            options,
            block_comment_depth,
            in_block_comment,
            count,
        );
        
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
    process_nesting_block_comment_line_with_options(
        line,
        &StringSkipOptions::rust(),
        block_comment_depth,
        in_block_comment,
        count,
    )
}

/// ネストされたブロックコメント行を処理 - StringSkipOptions対応版
fn process_nesting_block_comment_line_with_options(
    line: &str,
    options: &StringSkipOptions,
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
                        process_nesting_c_style_with_options(
                            rest,
                            options,
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
