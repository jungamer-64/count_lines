// crates/infra/src/measurement/strategies/sloc_counter/processors/matlab_style.rs
//! MATLAB / Octave のコメント処理
//!
//! MATLAB固有の対応:
//! - `%` で始まる行コメント
//! - `%{` ～ `%}` ブロックコメント (行頭必須)

/// MATLAB スタイル (% と %{ %}) の処理
pub fn process_matlab_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if line.trim() == "%}" {
            *in_block_comment = false;
        }
        return;
    }

    if line.trim() == "%{" {
        *in_block_comment = true;
        return;
    }

    if line.starts_with('%') {
        return;
    }

    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    // テストヘルパー: 複数行を処理
    fn process_lines(lines: &[&str]) -> usize {
        let mut count = 0;
        let mut in_block = false;
        for line in lines {
            process_matlab_style(line, &mut in_block, &mut count);
        }
        count
    }

    #[test]
    fn test_percent_comment() {
        let count = process_lines(&[
            "% comment",
            "x = 1;",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment() {
        let count = process_lines(&[
            "%{",
            "  block comment",
            "%}",
            "y = 2;",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_code_line() {
        let count = process_lines(&[
            "z = 3;",
        ]);
        assert_eq!(count, 1);
    }
}
