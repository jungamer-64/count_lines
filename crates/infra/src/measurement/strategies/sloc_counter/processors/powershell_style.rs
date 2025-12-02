// crates/infra/src/measurement/strategies/sloc_counter/processors/powershell_style.rs
//! PowerShell のコメント処理
//!
//! PowerShell は `#` 行コメントと `<# #>` ブロックコメントを使用します。
//! Ruby の `x < # comment` のような構文と競合しないよう、独立した処理を行います。

use super::super::string_utils::find_hash_outside_string;

/// PowerShell スタイル (# と <# #>) の処理
///
/// - 行コメント: `#` から行末まで
/// - ブロックコメント: `<#` から `#>` まで（複数行可）
pub fn process_powershell_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // ブロックコメント内
    if *in_block_comment {
        if let Some(pos) = trimmed.find("#>") {
            *in_block_comment = false;
            // 閉じた後にコードがあるかチェック
            let rest = &trimmed[pos + 2..];
            if !rest.trim().is_empty() {
                // 残りの部分に行コメントがあるかチェック
                if let Some(hash_pos) = find_hash_outside_string(rest) {
                    let before_hash = &rest[..hash_pos];
                    if !before_hash.trim().is_empty() {
                        *count += 1;
                    }
                } else {
                    *count += 1;
                }
            }
        }
        return;
    }

    // 空行チェック
    if trimmed.is_empty() {
        return;
    }

    // ブロックコメント開始をチェック（<#）
    if let Some(block_start) = find_block_comment_start(line) {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty()
            && find_hash_outside_string(before.trim()).is_none_or(|p| p > 0);

        // ブロックコメントが同じ行で閉じるか
        let after_start = &line[block_start + 2..];
        if let Some(block_end) = after_start.find("#>") {
            // 同じ行で閉じる
            let after_close = &after_start[block_end + 2..];
            let has_code_after = !after_close.trim().is_empty()
                && find_hash_outside_string(after_close.trim()).is_none_or(|p| p > 0);

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
        return;
    }

    // 行コメント (#) をチェック - 文字列外で
    if let Some(hash_pos) = find_hash_outside_string(trimmed) {
        let before = &trimmed[..hash_pos];
        if before.trim().is_empty() {
            // コメントのみの行
            return;
        }
        // コメント前にコードがある
        *count += 1;
        return;
    }

    // コードがある行
    *count += 1;
}

/// `<#` を文字列外で検索
///
/// PowerShell では `< #` (スペースあり) は比較演算子 + コメントだが、
/// `<#` (スペースなし) はブロックコメント開始
fn find_block_comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i + 1 < bytes.len() {
        // 文字列: "..." または '...'
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            while i < bytes.len() {
                // PowerShell のエスケープは ` (バッククォート)
                if bytes[i] == b'`' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // <# の検出
        if bytes[i] == b'<' && bytes[i + 1] == b'#' {
            return Some(i);
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_powershell_style("# comment", &mut in_block, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_with_inline_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_powershell_style("$x = 1 # comment", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment_single_line() {
        let mut in_block = false;
        let mut count = 0;
        process_powershell_style("<# block comment #>", &mut in_block, &mut count);
        assert_eq!(count, 0);
        assert!(!in_block);
    }

    #[test]
    fn test_block_comment_multiline() {
        let mut in_block = false;
        let mut count = 0;

        process_powershell_style("<# start", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_powershell_style("middle", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_powershell_style("#>", &mut in_block, &mut count);
        assert!(!in_block);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_before_block_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_powershell_style("$x = 1 <# comment", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment_start_in_string() {
        let mut in_block = false;
        let mut count = 0;
        // 文字列内の <# はコメント開始ではない
        process_powershell_style(r#"$s = "<# not a comment""#, &mut in_block, &mut count);
        assert!(!in_block);
        assert_eq!(count, 1);
    }
}
