// crates/core/src/language/processors/erlang_style.rs
//! Erlang / LaTeX / Prolog 等のコメント処理
//!
//! 対象: Erlang, LaTeX (.tex, .sty, .bib), Prolog 等
//! コメント: `%` で始まる行

/// Erlang スタイル (%) の処理
#[cfg(test)]
fn process_erlang_style(line: &str, count: &mut usize) {
    if line.starts_with('%') {
        return;
    }
    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_comment() {
        let mut count = 0;
        process_erlang_style("% comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_line() {
        let mut count = 0;
        process_erlang_style("main() -> ok.", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_latex_comment() {
        let mut count = 0;
        process_erlang_style("% LaTeX comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_latex_code() {
        let mut count = 0;
        process_erlang_style("\\documentclass{article}", &mut count);
        assert_eq!(count, 1);
    }
}
