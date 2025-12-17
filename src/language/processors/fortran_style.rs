// src/language/processors/fortran_style.rs
//! Fortran言語のコメント処理
//!
//! Fortran固有の対応:
//! - `!` で始まるコメント (Fortran 90+)
//! - `C`, `c`, `*` で始まる固定形式コメント (Fortran 77)

/// Fortran スタイル (!) の処理
#[cfg(test)]
fn process_fortran_style(line: &str, count: &mut usize) {
    // Fortran: ! で始まるコメント、または C/c/* で始まる固定形式コメント
    if line.starts_with('!')
        || line.starts_with('C')
        || line.starts_with('c')
        || line.starts_with('*')
    {
        return;
    }
    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exclamation_comment() {
        let mut count = 0;
        process_fortran_style("! Fortran 90 comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_c_comment() {
        let mut count = 0;
        process_fortran_style("C Fixed format comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_lowercase_c_comment() {
        let mut count = 0;
        process_fortran_style("c lowercase C comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_asterisk_comment() {
        let mut count = 0;
        process_fortran_style("* Asterisk comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_line() {
        let mut count = 0;
        process_fortran_style("      PROGRAM HELLO", &mut count);
        assert_eq!(count, 1);
    }
}
