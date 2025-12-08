// crates/infra/src/measurement/strategies/sloc_counter/processors/powershell_style.rs
//! PowerShell のコメント処理
//!
//! PowerShell は `#` 行コメントと `<# #>` ブロックコメントを使用します。

use super::super::processor_trait::LineProcessor;
use super::super::string_utils::find_hash_outside_string;

/// PowerShell プロセッサ
pub struct PowerShellProcessor {
    in_block_comment: bool,
}

impl Default for PowerShellProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor for PowerShellProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl PowerShellProcessor {
    pub fn new() -> Self {
        Self {
            in_block_comment: false,
        }
    }

    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        if self.in_block_comment {
            if let Some(pos) = trimmed.find("#>") {
                self.in_block_comment = false;
                let rest = &trimmed[pos + 2..];
                if !rest.trim().is_empty() {
                    if let Some(hash_pos) = find_hash_outside_string(rest) {
                        let before_hash = &rest[..hash_pos];
                        if !before_hash.trim().is_empty() {
                            return 1;
                        }
                    } else {
                        return 1;
                    }
                }
            }
            return 0;
        }

        if trimmed.is_empty() {
            return 0;
        }

        if let Some(block_start) = find_block_comment_start(line) {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty()
                && find_hash_outside_string(before.trim()).is_none_or(|p| p > 0);

            let after_start = &line[block_start + 2..];
            if let Some(block_end) = after_start.find("#>") {
                let after_close = &after_start[block_end + 2..];
                let has_code_after = !after_close.trim().is_empty()
                    && find_hash_outside_string(after_close.trim()).is_none_or(|p| p > 0);
                return if has_code_before || has_code_after { 1 } else { 0 };
            } else {
                self.in_block_comment = true;
                return if has_code_before { 1 } else { 0 };
            }
        }

        if let Some(hash_pos) = find_hash_outside_string(trimmed) {
            let before = &trimmed[..hash_pos];
            if before.trim().is_empty() {
                return 0;
            }
            return 1;
        }

        1
    }
}

fn find_block_comment_start(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if b == b'"' && !in_single {
            in_double = !in_double;
        } else if b == b'\'' && !in_double {
            in_single = !in_single;
        } else if b == b'<' && !in_single && !in_double && i + 1 < bytes.len() && bytes[i + 1] == b'#' {
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
    fn test_powershell_processor_line_comment() {
        let mut p = PowerShellProcessor::new();
        assert_eq!(p.process("# comment"), 0);
        assert_eq!(p.process("$x = 1"), 1);
    }

    #[test]
    fn test_powershell_processor_block_comment() {
        let mut p = PowerShellProcessor::new();
        assert_eq!(p.process("<# start"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("middle"), 0);
        assert_eq!(p.process("#>"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("$y = 2"), 1);
    }

    #[test]
    fn test_powershell_processor_inline_comment() {
        let mut p = PowerShellProcessor::new();
        assert_eq!(p.process("$x = 1 # comment"), 1);
    }
}
