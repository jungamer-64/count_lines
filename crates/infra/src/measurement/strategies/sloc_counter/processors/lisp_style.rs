// crates/infra/src/measurement/strategies/sloc_counter/processors/lisp_style.rs
//! Lisp系言語のコメント処理
//!
//! 対象: Lisp, Scheme, Clojure, Emacs Lisp 等
//! コメント: `;` で始まる行

#[cfg(test)]
fn process_lisp_style(line: &str, count: &mut usize) {
    if line.starts_with(';') {
        return;
    }
    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semicolon_comment() {
        let mut count = 0;
        process_lisp_style("; comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_line() {
        let mut count = 0;
        process_lisp_style("(defun foo ())", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_multiple_semicolons() {
        let mut count = 0;
        process_lisp_style(";;; section comment", &mut count);
        assert_eq!(count, 0);
    }
}
