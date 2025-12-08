// crates/infra/src/measurement/strategies/sloc_counter/processors/assembly_style.rs
//! GAS (GNU Assembler) アセンブリ言語のコメント処理
//!
//! GAS固有の対応:
//! - 行コメント: `#` と `@`
//! - Cスタイルブロックコメント: `/* */`

use super::super::processor_trait::LineProcessor;

/// GAS Assembly プロセッサ
pub struct GasAssemblyProcessor {
    in_block_comment: bool,
}

impl Default for GasAssemblyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl LineProcessor for GasAssemblyProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl GasAssemblyProcessor {
    pub fn new() -> Self {
        Self { in_block_comment: false }
    }

    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        if self.in_block_comment {
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() {
                    return self.process(rest);
                }
            }
            return 0;
        }

        // ブロックコメント開始
        if let Some(pos) = line.find("/*") {
            let before = &line[..pos];
            let has_code_before = !before.trim().is_empty();
            let rest = &line[pos + 2..];
            if let Some(end_pos) = rest.find("*/") {
                let after = &rest[end_pos + 2..];
                if has_code_before || !after.trim().is_empty() {
                    return 1;
                }
                return 0;
            }
            self.in_block_comment = true;
            return if has_code_before { 1 } else { 0 };
        }

        // 行コメント: # または @
        if trimmed.starts_with('#') || trimmed.starts_with('@') {
            return 0;
        }

        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_processor_hash_comment() {
        let mut p = GasAssemblyProcessor::new();
        assert_eq!(p.process("# comment"), 0);
        assert_eq!(p.process("mov r0, r1"), 1);
    }

    #[test]
    fn test_gas_processor_at_comment() {
        let mut p = GasAssemblyProcessor::new();
        assert_eq!(p.process("@ comment"), 0);
        assert_eq!(p.process("ldr r0, [r1]"), 1);
    }

    #[test]
    fn test_gas_processor_block_comment() {
        let mut p = GasAssemblyProcessor::new();
        assert_eq!(p.process("/* start"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("middle"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("str r0, [r1]"), 1);
    }
}
