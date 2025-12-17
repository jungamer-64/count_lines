// src/language/processors/simple_hash_style.rs
//! シンプルな Hash スタイル (#) のコメント処理
//!
//! 対象: Shell, YAML, TOML, Dockerfile, Makefile, Config系など

use super::super::processor_trait::LineProcessor;

/// 単純な Hash コメントプロセッサ
#[derive(Default)]
pub struct SimpleHashProcessor {
    line_count: usize,
}

impl LineProcessor for SimpleHashProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }
}

impl SimpleHashProcessor {
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

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

        // # より前にコードがあるか
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

/// 単純な文字列 ("..." と '...') 外の # を検索
#[must_use]
pub fn find_hash_outside_simple_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // エスケープ処理
        if b == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }

        // 文字列の開始/終了
        if b == b'"' && !in_single {
            in_double = !in_double;
        } else if b == b'\'' && !in_double {
            in_single = !in_single;
        } else if b == b'#' && !in_single && !in_double {
            return Some(i);
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_hash_processor_comment() {
        let mut p = SimpleHashProcessor::default();
        assert_eq!(p.process("# comment"), 0);
    }

    #[test]
    fn test_simple_hash_processor_code() {
        let mut p = SimpleHashProcessor::default();
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_simple_hash_processor_inline_comment() {
        let mut p = SimpleHashProcessor::default();
        assert_eq!(p.process("x = 1 # comment"), 1);
    }

    #[test]
    fn test_simple_hash_processor_shebang() {
        let mut p = SimpleHashProcessor::default();
        assert_eq!(p.process("#!/bin/bash"), 0);
        assert_eq!(p.process("echo hello"), 1);
    }

    #[test]
    fn test_simple_hash_processor_hash_in_string() {
        let mut p = SimpleHashProcessor::default();
        assert_eq!(p.process(r#"s = "hello # world""#), 1);
        assert_eq!(p.process("t = 'foo # bar'"), 1);
    }
}
