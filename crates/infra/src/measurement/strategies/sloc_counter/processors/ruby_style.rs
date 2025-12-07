// crates/infra/src/measurement/strategies/sloc_counter/processors/ruby_style.rs
//! Ruby言語のコメント処理
//!
//! Ruby固有の対応:
//! - `#` 行コメント
//! - 埋め込みドキュメント: `=begin` ～ `=end` (行頭必須)

use super::simple_hash_style::find_hash_outside_simple_string;

/// Rubyプロセッサ
#[derive(Default)]
pub struct RubyProcessor {
    in_embedded_doc: bool,
}

impl RubyProcessor {
    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // 埋め込みドキュメント内
        if self.in_embedded_doc {
            if line.starts_with("=end") {
                self.in_embedded_doc = false;
            }
            return 0;
        }

        // 埋め込みドキュメント開始 (行頭必須)
        if line.starts_with("=begin") {
            self.in_embedded_doc = true;
            return 0;
        }

        // #で始まる行はコメント
        if trimmed.starts_with('#') {
            return 0;
        }

        // # より前にコードがあるか (標準的な文字列のみ考慮)
        if let Some(hash_pos) = find_hash_outside_simple_string(line) {
            let before = &line[..hash_pos];
            if !before.trim().is_empty() {
                return 1;
            }
            return 0;
        }

        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruby_processor_line_comment() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("# comment"), 0);
    }

    #[test]
    fn test_ruby_processor_code() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_ruby_processor_embedded_doc() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("x = 1"), 1);
        assert_eq!(p.process("=begin"), 0);
        assert_eq!(p.process("documentation"), 0);
        assert_eq!(p.process("=end"), 0);
        assert_eq!(p.process("y = 2"), 1);
    }

    #[test]
    fn test_ruby_processor_inline_comment() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("x = 1  # comment"), 1);
    }

    #[test]
    fn test_ruby_processor_shebang() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process("#!/usr/bin/env ruby"), 0);
        assert_eq!(p.process("puts 'Hello'"), 1);
    }

    #[test]
    fn test_ruby_processor_string_with_hash() {
        let mut p = RubyProcessor::default();
        assert_eq!(p.process(r#"s = "hello # world""#), 1);
        assert_eq!(p.process("t = 'foo # bar'"), 1);
    }
}
