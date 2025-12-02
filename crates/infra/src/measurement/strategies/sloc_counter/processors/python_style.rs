// crates/infra/src/measurement/strategies/sloc_counter/processors/python_style.rs
//! Python言語のコメント処理
//!
//! Python固有の対応:
//! - Docstring: `"""..."""` / `'''...'''`
//! - f-string: `f"..."`, `F"..."` 等の文字列プレフィックス
//! - 複合プレフィックス: `fr"..."`, `rf"..."` 等
//! - shebang行の除外

use super::super::string_utils::{check_docstring_start, find_hash_outside_string};

/// Python スタイル (#) の処理
/// 
/// Python固有の対応:
/// - Docstring: `"""..."""` / `'''...'''`
/// - f-string: `f"..."`, `F"..."` 等の文字列プレフィックス
/// - 複合プレフィックス: `fr"..."`, `rf"..."` 等
pub fn process_python_style(
    line: &str,
    docstring_quote: &mut Option<u8>,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // Docstring内の場合
    if let Some(quote) = *docstring_quote {
        let closing = if quote == b'"' { "\"\"\"" } else { "'''" };
        if line.contains(closing) {
            *docstring_quote = None;
            *in_block_comment = false;
        }
        return;
    }

    // shebang行を除外
    if trimmed.starts_with("#!") && *count == 0 {
        return;
    }
    
    // #で始まる行はコメント
    if trimmed.starts_with('#') {
        return;
    }

    // Python Docstring開始判定（行頭または代入の右辺として現れる三重クォート）
    if let Some(quote_type) = check_docstring_start(trimmed) {
        let closing = if quote_type == b'"' { "\"\"\"" } else { "'''" };
        // 同じ行で閉じているか確認
        if trimmed.len() > 3 && trimmed[3..].contains(closing) {
            // 1行Docstring -> コメント扱い
            return;
        }
        *docstring_quote = Some(quote_type);
        *in_block_comment = true;
        return;
    }

    // # より前にコードがあるか (f-string等を考慮)
    if let Some(hash_pos) = find_hash_outside_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // テストヘルパー: 単一行を処理
    fn process_single_line(line: &str, count: &mut usize) {
        let mut docstring_quote: Option<u8> = None;
        let mut in_block: bool = false;
        process_python_style(line, &mut docstring_quote, &mut in_block, count);
    }

    // テストヘルパー: 複数行を処理
    fn process_lines(lines: &[&str]) -> usize {
        let mut count = 0;
        let mut docstring_quote: Option<u8> = None;
        let mut in_block: bool = false;
        for line in lines {
            process_python_style(line, &mut docstring_quote, &mut in_block, &mut count);
        }
        count
    }

    #[test]
    fn test_hash_comment() {
        let mut count = 0;
        process_single_line("# comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_code_line() {
        let mut count = 0;
        process_single_line("x = 1", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_inline_comment() {
        let mut count = 0;
        process_single_line("x = 1  # comment", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_shebang() {
        let count = process_lines(&[
            "#!/usr/bin/env python",
            "x = 1",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_docstring_multiline() {
        let count = process_lines(&[
            "def foo():",
            "    \"\"\"",
            "    This is a docstring.",
            "    Multiple lines.",
            "    \"\"\"",
            "    return 1",
        ]);
        // def foo(): と return 1 のみがSLOC
        assert_eq!(count, 2);
    }

    #[test]
    fn test_docstring_single_line() {
        let count = process_lines(&[
            "def bar():",
            "    \"\"\"Single line docstring.\"\"\"",
            "    pass",
        ]);
        // def bar(): と pass のみがSLOC
        assert_eq!(count, 2);
    }

    #[test]
    fn test_docstring_single_quote() {
        let count = process_lines(&[
            "'''",
            "Triple single quote docstring",
            "'''",
            "x = 1",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_string_with_hash() {
        // 文字列内の # はコメントではない
        let count = process_lines(&[
            "s = \"hello # world\"",
            "t = 'foo # bar'",
        ]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_fstring_with_hash() {
        let count = process_lines(&[
            r#"s = f"Hash: #{value}""#,
            "x = 1",
        ]);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_various_prefixes() {
        let count = process_lines(&[
            r#"a = f"test # not comment""#,
            r#"b = F"test # not comment""#,
            r#"c = r"test # not comment""#,
            r#"d = u"test # not comment""#,
            r#"e = b"test # not comment""#,
            r#"f = fr"test # not comment""#,
            r#"g = rf"test # not comment""#,
        ]);
        // 全て7行がSLOC
        assert_eq!(count, 7);
    }

    #[test]
    fn test_fstring_multiline() {
        let count = process_lines(&[
            r#"s = f"""Multi"#,
            "line # not comment",
            r#"string""""#,
            "x = 1",
        ]);
        // 三重引用符の開始行、中間行、終了行、x = 1 の4行がSLOC
        assert_eq!(count, 4);
    }
}
