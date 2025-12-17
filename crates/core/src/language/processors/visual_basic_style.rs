// src/language/processors/visual_basic_style.rs
//! `Visual Basic` / `VBA` / `VBScript` のコメント処理
//!
//! `VB`系言語固有の対応:
//! - `'` で始まる行コメント
//! - `REM` で始まる行コメント (大文字小文字不問)
//! - 行中の `'` 以降もコメント（文字列リテラル外）
//! - `""` でエスケープされたダブルクォート

/// Visual Basic / VBA / `VBScript` スタイル (' と REM) の処理
///
/// VB系言語のコメント:
/// - `'` で始まる行コメント
/// - `REM` で始まる行コメント (大文字小文字不問)
/// - 行中の `'` 以降もコメント（文字列リテラル外）
#[cfg(test)]
fn process_visual_basic_style(line: &str, count: &mut usize) {
    let trimmed = line.trim();

    // ' で始まるコメント行
    if trimmed.starts_with('\'') {
        return;
    }

    // REM コメント (大文字小文字不問)
    let upper = trimmed.to_uppercase();
    if upper == "REM" || upper.starts_with("REM ") || upper.starts_with("REM\t") {
        return;
    }

    // 文字列リテラル外の ' を探す
    // VBの文字列は "" でエスケープ、\ はエスケープなし
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_string = false;

    while i < bytes.len() {
        if in_string {
            if bytes[i] == b'"' {
                // "" はエスケープされた "
                if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                    i += 2;
                    continue;
                }
                in_string = false;
            }
            i += 1;
            continue;
        }

        if bytes[i] == b'"' {
            in_string = true;
            i += 1;
            continue;
        }

        if bytes[i] == b'\'' {
            // ' 以前にコードがあればカウント
            let before = &line[..i];
            if !before.trim().is_empty() {
                *count += 1;
            }
            return;
        }

        i += 1;
    }

    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_quote_comment() {
        let mut count = 0;
        process_visual_basic_style("' This is a VB comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_rem_comment() {
        let mut count = 0;
        process_visual_basic_style("REM This is a REM comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_line() {
        let mut count = 0;
        process_visual_basic_style("Dim x As Integer", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_inline_comment() {
        let mut count = 0;
        process_visual_basic_style("Dim y As String ' variable declaration", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_string_with_quote() {
        let mut count = 0;
        process_visual_basic_style("s = \"It's a test\" ' comment", &mut count);
        // 1行がSLOC（文字列内の ' はコメントではない）
        assert_eq!(count, 1);
    }

    #[test]
    fn test_escaped_quote_in_string() {
        let mut count = 0;
        process_visual_basic_style("s = \"He said \"\"Hello\"\"\" ' comment", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_rem_lowercase() {
        let mut count = 0;
        process_visual_basic_style("rem lowercase comment", &mut count);
        assert_eq!(count, 0);
    }
}
