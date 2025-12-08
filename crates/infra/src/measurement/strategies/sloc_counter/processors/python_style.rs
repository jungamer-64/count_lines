// crates/infra/src/measurement/strategies/sloc_counter/processors/python_style.rs
//! Python言語のコメント処理
//!
//! Python固有の対応:
//! - Docstring: `"""..."""` / `'''...'''`
//! - f-string: `f"..."`, `F"..."` 等の文字列プレフィックス
//! - 複合プレフィックス: `fr"..."`, `rf"..."` 等
//! - shebang行の除外

use super::super::string_utils::{check_docstring_start, find_hash_outside_string};
use super::super::processor_trait::LineProcessor;

/// Pythonプロセッサ
#[derive(Default)]
pub struct PythonProcessor {
    docstring_quote: Option<u8>,
    line_count: usize,
}

impl LineProcessor for PythonProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.docstring_quote.is_some()
    }
}

impl PythonProcessor {
    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // Docstring内の場合
        if let Some(quote) = self.docstring_quote {
            let closing = if quote == b'"' { "\"\"\"" } else { "'''" };
            if line.contains(closing) {
                self.docstring_quote = None;
            }
            return 0;
        }

        // shebang行を除外 (最初の行のみ)
        if trimmed.starts_with("#!") && self.line_count == 0 {
            self.line_count += 1;
            return 0;
        }
        self.line_count += 1;
        
        // #で始まる行はコメント
        if trimmed.starts_with('#') {
            return 0;
        }

        // Python Docstring開始判定（行頭または代入の右辺として現れる三重クォート）
        if let Some(quote_type) = check_docstring_start(trimmed) {
            let closing = if quote_type == b'"' { "\"\"\"" } else { "'''" };
            // 同じ行で閉じているか確認
            if trimmed.len() > 3 && trimmed[3..].contains(closing) {
                // 1行Docstring -> コメント扱い
                return 0;
            }
            self.docstring_quote = Some(quote_type);
            return 0;
        }

        // # より前にコードがあるか (f-string等を考慮)
        if let Some(hash_pos) = find_hash_outside_string(line) {
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
    fn test_python_processor_hash_comment() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("# comment"), 0);
    }

    #[test]
    fn test_python_processor_code() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_python_processor_inline_comment() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = 1  # comment"), 1);
    }

    #[test]
    fn test_python_processor_docstring() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("def foo():"), 1);
        assert_eq!(p.process("    \"\"\""), 0);
        assert_eq!(p.process("    Docstring"), 0);
        assert_eq!(p.process("    \"\"\""), 0);
        assert_eq!(p.process("    return 1"), 1);
    }

    #[test]
    fn test_python_processor_shebang() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("#!/usr/bin/env python"), 0);
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_python_processor_single_quote_docstring() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("'''"), 0);
        assert_eq!(p.process("Triple single quote docstring"), 0);
        assert_eq!(p.process("'''"), 0);
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_python_processor_string_with_hash() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process(r#"s = "hello # world""#), 1);
        assert_eq!(p.process("t = 'foo # bar'"), 1);
    }

    #[test]
    fn test_python_processor_fstring_with_hash() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process(r#"s = f"Hash: #{value}""#), 1);
    }
}
