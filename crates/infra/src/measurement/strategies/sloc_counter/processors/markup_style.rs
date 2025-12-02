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
