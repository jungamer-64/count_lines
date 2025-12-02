// crates/infra/src/measurement/strategies/sloc_counter/processors/markup_style.rs
//! マークアップ言語のコメント処理
//!
//! HTML/XML/SVG などの <!-- --> コメントを処理します。

/// HTML スタイル (<!-- -->) の処理
pub fn process_html_style(line: &str, in_block_comment: &mut bool, count: &mut usize) {
    if *in_block_comment {
        if line.contains("-->") {
            *in_block_comment = false;
            // --> の後にコードがあるかチェック
            if let Some(pos) = line.find("-->") {
                let rest = &line[pos + 3..];
                if !rest.trim().is_empty() {
                    *count += 1;
                }
            }
        }
        return;
    }

    if let Some(start) = line.find("<!--") {
        let before = &line[..start];
        let has_code_before = !before.trim().is_empty();

        if let Some(end_offset) = line[start + 4..].find("-->") {
            let after = &line[start + 4 + end_offset + 3..];
            if has_code_before || !after.trim().is_empty() {
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

    *count += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_comment_single_line() {
        let mut in_block = false;
        let mut count = 0;
        process_html_style("<!-- comment -->", &mut in_block, &mut count);
        assert_eq!(count, 0);
        assert!(!in_block);
    }

    #[test]
    fn test_html_code() {
        let mut in_block = false;
        let mut count = 0;
        process_html_style("<div>content</div>", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_html_multiline_comment() {
        let mut in_block = false;
        let mut count = 0;

        process_html_style("<!-- multi", &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_html_style("line -->", &mut in_block, &mut count);
        assert!(!in_block);
        assert_eq!(count, 0);

        process_html_style("<p>text</p>", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_html_code_before_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_html_style("<div>content</div> <!-- comment -->", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_html_code_after_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_html_style("<!-- comment --> <div>content</div>", &mut in_block, &mut count);
        assert_eq!(count, 1);
    }
}
