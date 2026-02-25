// crates/core/src/language/processors/python_style.rs
//! # Python Processor
//!
//! SLOC counter processor for Python with special handling for docstrings and f-strings.
//!
//! ## Supported Syntax
//!
//! - **Line comments**: `#`
//! - **Docstrings**: `"""..."""` and `'''...'''` (treated as comments when standalone)
//! - **f-strings**: `f"..."`, `F"..."` with `{expr}` interpolation
//! - **Raw strings**: `r"..."`, `R"..."`
//! - **Byte strings**: `b"..."`, `B"..."`
//! - **Combined prefixes**: `rf"..."`, `fr"..."`, `br"..."`, etc.
//! - **Shebang**: `#!/usr/bin/env python` (first line only)
//!
//! ## Docstring Detection
//!
//! Triple-quoted strings are treated as docstrings (comments) when they appear
//! at the start of a line with no preceding code. Otherwise, they are counted as code.
//!
//! ## Performance Characteristics
//!
//! - **Time complexity**: O(n) where n = line length
//! - **Space complexity**: O(d) where d = nesting depth (f-string interpolation)
//! - **Thread safety**: No (has internal mutable state)
//!
//! ## Usage Example
//!
//! ```rust
//! use count_lines_core::language::processors::PythonProcessor;
//! use count_lines_core::language::processor_trait::LineProcessor;
//!
//! let mut proc = PythonProcessor::new();
//!
//! // Code line
//! assert_eq!(proc.process_line("x = 1"), 1);
//!
//! // Comment line
//! assert_eq!(proc.process_line("# this is a comment"), 0);
//!
//! // Docstring (standalone triple-quote)
//! proc.reset();
//! assert_eq!(proc.process_line("\"\"\""), 0);
//! assert_eq!(proc.process_line("Docstring content"), 0);
//! assert_eq!(proc.process_line("\"\"\""), 0);
//! ```

use core::iter::Peekable;
use core::str::CharIndices;

use alloc::vec::Vec;

use super::super::processor_trait::LineProcessor;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Python scope state.
pub enum PythonScope {
    /// F-string interpolation scope.
    Interpolation, // { ... }
    /// String literal scope.
    String(PythonStringState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
/// Python string literal state.
pub struct PythonStringState {
    /// Quote character (`"` or `'`).
    pub quote: u8,    // " or '
    /// Whether this is a triple-quoted string.
    pub triple: bool, // """ or '''
    /// Whether this is an f-string.
    pub is_f_string: bool,
    /// Whether this is a raw string.
    pub is_raw: bool,
    /// Whether this string is a doc comment.
    pub is_doc_comment: bool, // Treat content as comment?
}

/// 文字列プレフィックスの解析結果
#[derive(Debug, Default)]
struct PrefixParseResult {
    is_f_string: bool,
    is_raw: bool,
    quote: Option<u8>,
}

/// Pythonプロセッサ
#[derive(Default, Clone, Debug)]
pub struct PythonProcessor {
    stack: Vec<PythonScope>,
    line_count: usize,
}

impl LineProcessor for PythonProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        // Docstring (triple quoted string) acting as comment
        if let Some(PythonScope::String(state)) = self.stack.last() {
            return state.is_doc_comment;
        }
        false
    }
}

impl PythonProcessor {
    #[must_use]
    /// Creates a new `PythonProcessor`.
    pub fn new() -> Self {
        Self::default()
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // shebang行を除外 (最初の行のみ)
        if trimmed.starts_with("#!") && self.line_count == 0 {
            self.line_count += 1;
            return 0;
        }
        self.line_count += 1;

        // Stack state determines initial has_code_token
        let mut has_code_token = if let Some(PythonScope::String(state)) = self.stack.last() {
            !state.is_doc_comment // Continuation of data string
        } else {
            false
        };

        let mut chars = line.char_indices().peekable();

        while let Some((_, c)) = chars.next() {
            match self.stack.last().cloned() {
                Some(PythonScope::String(state)) => {
                    has_code_token = self.handle_string_char(&mut chars, &state, c, has_code_token);
                }
                Some(PythonScope::Interpolation) => {
                    has_code_token = self.handle_interpolation_char(&mut chars, c, has_code_token);
                }
                None => {
                    has_code_token = self.handle_code_char(&mut chars, c, has_code_token);
                }
            }
        }

        usize::from(has_code_token)
    }

    /// 文字列内の文字を処理
    fn handle_string_char(
        &mut self,
        chars: &mut Peekable<CharIndices<'_>>,
        state: &PythonStringState,
        c: char,
        mut has_code_token: bool,
    ) -> bool {
        let quote_char = state.quote as char;

        // Check for string end
        if c == quote_char {
            if Self::check_string_end(chars, state) {
                self.stack.pop();
                if !state.is_doc_comment {
                    has_code_token = true;
                }
                return has_code_token;
            }
            // Not ended - treat as regular char
            if !state.is_doc_comment && !c.is_whitespace() {
                has_code_token = true;
            }
            return has_code_token;
        }

        // Escape handling
        if c == '\\' {
            has_code_token = Self::handle_escape(chars, state, has_code_token);
            return has_code_token;
        }

        // Interpolation handling (f-string)
        if state.is_f_string && c == '{' {
            has_code_token = self.handle_interpolation_start(chars, has_code_token);
            return has_code_token;
        }

        if !state.is_doc_comment && !c.is_whitespace() {
            has_code_token = true;
        }

        has_code_token
    }

    /// 文字列終端のチェック
    /// Returns true if string ended, false if quote matched but not triple end.
    fn check_string_end(chars: &mut Peekable<CharIndices<'_>>, state: &PythonStringState) -> bool {
        let quote_char = state.quote as char;

        if state.triple {
            // Check next 2 chars for triple quote
            if chars.peek().is_some_and(|&(_, c2)| c2 == quote_char) {
                chars.next(); // consume 2nd
                if chars.peek().is_some_and(|&(_, c3)| c3 == quote_char) {
                    chars.next(); // consume 3rd
                    return true; // Triple quote end
                }
                // Only 2 quotes - not end of triple
                return false;
            }
            false
        } else {
            // Single quote end
            true
        }
    }

    /// エスケープ文字の処理
    fn handle_escape(
        chars: &mut Peekable<CharIndices<'_>>,
        state: &PythonStringState,
        mut has_code_token: bool,
    ) -> bool {
        let quote_char = state.quote as char;

        if state.is_raw {
            // Raw string: backslash is literal, but may escape quote
            if chars
                .peek()
                .is_some_and(|&(_, next_c)| next_c == quote_char)
            {
                chars.next(); // Consume quote as escaped
            }
        } else {
            // Normal string: escape next char
            chars.next();
        }

        if !state.is_doc_comment {
            has_code_token = true;
        }

        has_code_token
    }

    /// f-string内の補間開始処理
    fn handle_interpolation_start(
        &mut self,
        chars: &mut Peekable<CharIndices<'_>>,
        mut has_code_token: bool,
    ) -> bool {
        // Check for {{ (escape)
        if chars.peek().is_some_and(|&(_, next_c)| next_c == '{') {
            chars.next(); // consume second {
        // Escaped brace, not interpolation
        } else {
            self.stack.push(PythonScope::Interpolation);
            has_code_token = true;
        }

        has_code_token
    }

    /// 補間スコープ内の文字処理
    fn handle_interpolation_char(
        &mut self,
        chars: &mut Peekable<CharIndices<'_>>,
        c: char,
        has_code_token: bool,
    ) -> bool {
        if c == '}' {
            self.stack.pop();
            return true; // } is code
        }

        // Interpolation can contain nested strings
        self.handle_code_char(chars, c, has_code_token)
    }

    /// コードスコープ内の文字処理
    fn handle_code_char(
        &mut self,
        chars: &mut Peekable<CharIndices<'_>>,
        c: char,
        mut has_code_token: bool,
    ) -> bool {
        // Check comment
        if c == '#' {
            // Comment starts - stop processing this line
            // Return current has_code_token state (don't set to true for comment)
            // We need to drain the rest of the iterator
            while chars.next().is_some() {}
            return has_code_token;
        }

        // Check for string prefix or quote
        let lower_c = c.to_ascii_lowercase();
        let is_prefix = matches!(lower_c, 'f' | 'r' | 'u' | 'b');

        if c == '"' || c == '\'' || is_prefix {
            if let Some(string_state) = Self::try_parse_string_start(chars, c, has_code_token) {
                self.stack.push(PythonScope::String(string_state.clone()));
                if !string_state.is_doc_comment {
                    has_code_token = true;
                }
                return has_code_token;
            }
            // Failed to parse string - treat as identifier/code
            has_code_token = true;
        } else if !c.is_whitespace() {
            has_code_token = true;
        }

        // Check closing } for interpolation (handled in handle_interpolation_char)

        has_code_token
    }

    /// 文字列開始のパース試行
    fn try_parse_string_start(
        chars: &mut Peekable<CharIndices<'_>>,
        first_char: char,
        has_code_token: bool,
    ) -> Option<PythonStringState> {
        let prefix = Self::parse_prefix(chars, first_char)?;
        let quote = prefix.quote?;

        // Check for triple quote
        let quote_char = quote as char;
        let triple = if chars.peek().is_some_and(|&(_, c2)| c2 == quote_char) {
            chars.next(); // consume 2nd
            if chars.peek().is_some_and(|&(_, c3)| c3 == quote_char) {
                chars.next(); // consume 3rd
                true
            } else {
                // "" or '' - empty string, already consumed
                return None; // Empty string handled inline
            }
        } else {
            false
        };

        // Docstring: only if triple and no code seen yet on this line
        let is_doc_comment = triple && !has_code_token;

        Some(PythonStringState {
            quote,
            triple,
            is_f_string: prefix.is_f_string,
            is_raw: prefix.is_raw,
            is_doc_comment,
        })
    }

    /// 文字列プレフィックスのパース
    fn parse_prefix(
        chars: &mut Peekable<CharIndices<'_>>,
        first_char: char,
    ) -> Option<PrefixParseResult> {
        let mut result = PrefixParseResult::default();
        let mut current = first_char;

        // Track seen prefixes
        let mut seen_f = false;
        let mut seen_r = false;
        let mut seen_b = false;
        let mut seen_u = false;

        loop {
            if current == '"' || current == '\'' {
                result.quote = Some(current as u8);
                return Some(result);
            }

            let lc = current.to_ascii_lowercase();
            match lc {
                'f' if !seen_f => {
                    seen_f = true;
                    result.is_f_string = true;
                }
                'r' if !seen_r => {
                    seen_r = true;
                    result.is_raw = true;
                }
                'b' if !seen_b => {
                    seen_b = true;
                }
                'u' if !seen_u => {
                    seen_u = true;
                }
                _ => return None, // Invalid prefix sequence
            }

            // Get next char
            current = chars.next()?.1;
        }
    }
}

// ============================================================================
// StatefulProcessor implementation
// ============================================================================

use super::super::processor_trait::StatefulProcessor;

/// State for `PythonProcessor`.
#[derive(Debug, Clone, Default)]
pub struct PythonState {
    /// Current scope stack (strings, interpolations).
    pub stack: Vec<PythonScope>,
    /// Number of lines processed (for shebang detection).
    pub line_count: usize,
}

impl StatefulProcessor for PythonProcessor {
    type State = PythonState;

    fn get_state(&self) -> Self::State {
        PythonState {
            stack: self.stack.clone(),
            line_count: self.line_count,
        }
    }

    fn set_state(&mut self, state: Self::State) {
        self.stack = state.stack;
        self.line_count = state.line_count;
    }

    fn is_in_multiline_context(&self) -> bool {
        // In a multi-line string or interpolation
        !self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_python_processor_raw_docstring() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("r\"\"\""), 0);
        assert_eq!(p.process("Raw Doc"), 0);
        assert_eq!(p.process("\"\"\""), 0);
    }

    #[test]
    fn test_python_processor_f_string() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = f\"val { 1 }\""), 1);
    }

    #[test]
    fn test_python_processor_f_string_nested() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = f\"val { f'{ 2 }' }\""), 1);
    }

    #[test]
    fn test_python_multiline_f_string_comment() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = f\"start {"), 1);
        assert_eq!(p.process("  # comment"), 0);
        assert_eq!(p.process("  y"), 1);
        assert_eq!(p.process("}\""), 1);
    }

    #[test]
    fn test_python_single_line_comment() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("# just a comment"), 0);
        assert_eq!(p.process("x = 1  # inline comment"), 1);
    }

    #[test]
    fn test_python_empty_string() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = \"\""), 1);
        assert_eq!(p.process("y = ''"), 1);
    }

    #[test]
    fn test_python_escaped_quote() {
        let mut p = PythonProcessor::default();
        assert_eq!(p.process("x = \"hello \\\"world\\\"\""), 1);
    }
}
