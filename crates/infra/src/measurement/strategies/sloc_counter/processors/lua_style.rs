// crates/infra/src/measurement/strategies/sloc_counter/processors/lua_style.rs
//! Lua言語のコメント処理
//!
//! Lua固有の対応:
//! - 行コメント: `--`
//! - ブロックコメント: `--[[` ～ `]]`
//! - 等号付きブロックコメント: `--[=[` ～ `]=]`, `--[==[` ～ `]==]` 等

use super::super::processor_trait::LineProcessor;

/// Luaプロセッサ
#[derive(Default)]
pub struct LuaProcessor {
    in_block_comment: bool,
    block_level: usize,
}

impl LineProcessor for LuaProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

impl LuaProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        if self.in_block_comment {
            if find_lua_block_end(line, self.block_level).is_some() {
                self.in_block_comment = false;
                self.block_level = 0;
            }
            return 0;
        }

        // 行コメント
        if line.starts_with("--") {
            if let Some(level) = check_lua_block_start(&line[2..]) {
                let after_open = skip_lua_block_open(&line[2..]);
                if find_lua_block_end(after_open, level).is_some() {
                    return 0;
                }
                self.in_block_comment = true;
                self.block_level = level;
            }
            return 0;
        }

        1
    }
}

fn check_lua_block_start(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return None;
    }
    let mut i = 1;
    let mut level = 0;
    while i < bytes.len() && bytes[i] == b'=' {
        level += 1;
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'[' {
        return Some(level);
    }
    None
}

fn skip_lua_block_open(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return s;
    }
    let mut i = 1;
    while i < bytes.len() && bytes[i] == b'=' {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'[' {
        return &s[i + 1..];
    }
    s
}

fn find_lua_block_end(s: &str, level: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b']' {
            let mut eq_count = 0;
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b'=' {
                eq_count += 1;
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b']' && eq_count == level {
                return Some(j + 1);
            }
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_processor_line_comment() {
        let mut p = LuaProcessor::new();
        assert_eq!(p.process("-- comment"), 0);
        assert_eq!(p.process("local x = 1"), 1);
    }

    #[test]
    fn test_lua_processor_block_comment() {
        let mut p = LuaProcessor::new();
        assert_eq!(p.process("--[["), 0);
        assert_eq!(p.process("block comment"), 0);
        assert_eq!(p.process("]]"), 0);
        assert_eq!(p.process("local y = 2"), 1);
    }

    #[test]
    fn test_lua_processor_level_block() {
        let mut p = LuaProcessor::new();
        assert_eq!(p.process("--[=["), 0);
        assert_eq!(p.process("contains ]] but not end"), 0);
        assert_eq!(p.process("]=]"), 0);
        assert_eq!(p.process("local z = 3"), 1);
    }

    #[test]
    fn test_lua_processor_single_line_block() {
        let mut p = LuaProcessor::new();
        assert_eq!(p.process("--[[ single line block ]]"), 0);
        assert_eq!(p.process("local a = 1"), 1);
    }
}
