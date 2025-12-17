// crates/infra/src/measurement/strategies/sloc_counter/processors/batch_style.rs
//! Windows バッチファイルのコメント処理
//!
//! バッチファイル固有の対応:
//! - `REM` (大文字小文字不問) で始まる行
//! - `::` で始まる行 (ラベルの特殊用法としてのコメント)
//! - `@REM` で始まる行

/// Batch スタイル (REM と ::) の処理
///
/// Windows バッチファイルのコメント:
/// - `REM` (大文字小文字不問) で始まる行
/// - `::` で始まる行 (ラベルの特殊用法としてのコメント)
#[cfg(test)]
fn process_batch_style(line: &str, count: &mut usize) {
    let trimmed = line.trim();

    // REM コメント (大文字小文字不問)
    // "REM" の後にスペースか行末が必要
    let upper = trimmed.to_uppercase();
    if upper == "REM" || upper.starts_with("REM ") || upper.starts_with("REM\t") {
        return;
    }

    // :: コメント (ラベルの特殊用法)
    if trimmed.starts_with("::") {
        return;
    }

    // @ プレフィックス付きの REM
    if let Some(stripped) = trimmed.strip_prefix('@') {
        let after_at = stripped.trim_start();
        let upper_after = after_at.to_uppercase();
        if upper_after == "REM" || upper_after.starts_with("REM ") || upper_after.starts_with("REM\t") {
            return;
        }
    }

    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rem_comment() {
        let mut count = 0;
        process_batch_style("REM This is a comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_rem_lowercase() {
        let mut count = 0;
        process_batch_style("rem lowercase comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_double_colon_comment() {
        let mut count = 0;
        process_batch_style(":: This is a label comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_at_rem() {
        let mut count = 0;
        process_batch_style("@REM Suppress output and comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_line() {
        let mut count = 0;
        process_batch_style("echo Hello", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_rem_only() {
        let mut count = 0;
        process_batch_style("REM", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_not_rem_if_no_space() {
        // "REMARK" は REM コメントではない
        let mut count = 0;
        process_batch_style("echo REMARK", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_rem_with_tab() {
        let mut count = 0;
        process_batch_style("REM\tcomment with tab", &mut count);
        assert_eq!(count, 0);
    }
}
