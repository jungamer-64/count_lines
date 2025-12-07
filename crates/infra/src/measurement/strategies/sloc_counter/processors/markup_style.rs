// crates/infra/src/measurement/strategies/sloc_counter/processors/markup_style.rs
//! マークアップ言語のコメント処理
//!
//! HTML/XML/SVG などの <!-- --> コメントを処理します。

/// HTML/XML プロセッサ
///
/// `<!-- -->` コメントを処理します。
pub struct HtmlProcessor {
    in_comment: bool,
}

impl HtmlProcessor {
    pub fn new() -> Self {
        Self { in_comment: false }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        if self.in_comment {
            if line.contains("-->") {
                self.in_comment = false;
                if let Some(pos) = line.find("-->") {
                    let rest = &line[pos + 3..];
                    if !rest.trim().is_empty() {
                        return 1;
                    }
                }
            }
            return 0;
        }

        if let Some(start) = line.find("<!--") {
            let before = &line[..start];
            let has_code_before = !before.trim().is_empty();

            if let Some(end_offset) = line[start + 4..].find("-->") {
                let after = &line[start + 4 + end_offset + 3..];
                return if has_code_before || !after.trim().is_empty() { 1 } else { 0 };
            } else {
                self.in_comment = true;
                return if has_code_before { 1 } else { 0 };
            }
        }

        1
    }

    #[cfg(test)]
    pub fn is_in_comment(&self) -> bool {
        self.in_comment
    }
}

impl Default for HtmlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_processor_comment() {
        let mut p = HtmlProcessor::new();
        assert_eq!(p.process("<!-- comment -->"), 0);
    }

    #[test]
    fn test_html_processor_code() {
        let mut p = HtmlProcessor::new();
        assert_eq!(p.process("<div>content</div>"), 1);
    }

    #[test]
    fn test_html_processor_multiline_comment() {
        let mut p = HtmlProcessor::new();
        assert_eq!(p.process("<!-- start"), 0);
        assert!(p.is_in_comment());
        assert_eq!(p.process("middle"), 0);
        assert_eq!(p.process("end -->"), 0);
        assert!(!p.is_in_comment());
        assert_eq!(p.process("<p>text</p>"), 1);
    }

    #[test]
    fn test_html_processor_code_before_comment() {
        let mut p = HtmlProcessor::new();
        assert_eq!(p.process("<div>content</div> <!-- comment -->"), 1);
    }

    #[test]
    fn test_html_processor_code_after_comment() {
        let mut p = HtmlProcessor::new();
        assert_eq!(p.process("<!-- comment --> <div>content</div>"), 1);
    }
}
