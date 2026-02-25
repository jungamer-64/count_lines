// src/language/processors/javascript_style.rs
//! # JavaScript/TypeScript Processor
//!
//! SLOC counter processor for JavaScript, TypeScript, and related languages.
//!
//! ## Supported Syntax
//!
//! - **Line comments**: `//`
//! - **Block comments**: `/* */`
//! - **String literals**: `"..."`, `'...'`, `` `...` ``
//! - **Template literals**: `` `${...}` `` with interpolation
//! - **Regex literals**: `/pattern/flags`
//! - **Shebang**: `#!/usr/bin/env node` (first line only)
//!
//! ## Limitations
//!
//! - Nested comment-like syntax within regex patterns is not detected
//! - JSX/TSX special syntax (`<Component />`) is not fully supported
//! - Automatic semicolon insertion (ASI) edge cases may affect accuracy
//!
//! ## Performance Characteristics
//!
//! - **Time complexity**: O(n) where n = line length
//! - **Space complexity**: O(d) where d = maximum nesting depth
//! - **Thread safety**: No (has internal mutable state)
//!
//! ## Usage Example
//!
//! ```rust
//! use count_lines_core::language::processors::JavaScriptProcessor;
//! use count_lines_core::language::processor_trait::LineProcessor;
//!
//! let mut proc = JavaScriptProcessor::new();
//!
//! // Code line
//! assert_eq!(proc.process_line("let x = 1;"), 1);
//!
//! // Comment line
//! assert_eq!(proc.process_line("// this is a comment"), 0);
//!
//! // Regex literal (not a comment)
//! proc.reset();
//! assert_eq!(proc.process_line("const re = /pattern/;"), 1);
//! ```

use super::super::processor_trait::LineProcessor;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsScope {
    Interpolation, // ${ ... }
    BlockComment,  // /* ... */
    String(u8),    // " ' `
    Regex { in_class: bool },
}

#[derive(Default, Clone, Debug)]
pub struct JavaScriptProcessor {
    stack: Vec<JsScope>,
    // Track if last token was value-like (for regex heuristics)
    last_token_is_value: bool,
    line_count: usize,
}

impl LineProcessor for JavaScriptProcessor {
    fn process_line(&mut self, line: &str) -> usize {
        self.process(line)
    }

    fn is_in_block_comment(&self) -> bool {
        matches!(self.stack.last(), Some(JsScope::BlockComment))
    }
}

impl JavaScriptProcessor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // 1. Shebang check (start of file)
        if trimmed.starts_with("#!") && self.line_count == 0 {
            self.line_count += 1;
            return 0;
        }
        self.line_count += 1;

        let mut has_code_token = matches!(
            self.stack.last(),
            Some(JsScope::String(_) | JsScope::Regex { .. })
        );

        let mut chars = line.char_indices().peekable();

        while let Some((_, c)) = chars.next() {
            match self.stack.last().cloned() {
                Some(JsScope::BlockComment) => {
                    // Check for */
                    if c == '*' && chars.peek().is_some_and(|(_, next_c)| *next_c == '/') {
                        chars.next();
                        self.stack.pop();
                    }
                }
                Some(JsScope::String(quote)) => {
                    // Check escape
                    if c == '\\' {
                        chars.next(); // consume next
                        has_code_token = true;
                        continue;
                    }
                    // Check end check
                    if c == quote as char {
                        self.stack.pop();
                        self.last_token_is_value = true; // String literal is a value
                    }
                    // Check Interpolation ${ for backtick
                    else if quote == b'`'
                        && c == '$'
                        && chars.peek().is_some_and(|(_, next_c)| *next_c == '{')
                    {
                        chars.next();
                        self.stack.push(JsScope::Interpolation);
                        self.last_token_is_value = false; // Start of expr
                    }

                    if !c.is_whitespace() {
                        has_code_token = true;
                    }
                }
                Some(JsScope::Regex { in_class }) => {
                    // Regex literal handling
                    if c == '\\' {
                        // Escape next char (works in and out of class)
                        chars.next();
                        has_code_token = true;
                        continue;
                    }

                    if in_class {
                        if c == ']' {
                            // End class (replace top)
                            self.stack.pop(); // remove Regex { in_class: true }
                            self.stack.push(JsScope::Regex { in_class: false });
                        }
                    } else if c == '[' {
                        self.stack.pop();
                        self.stack.push(JsScope::Regex { in_class: true });
                    } else if c == '/' {
                        // End regex
                        self.stack.pop();
                        self.last_token_is_value = true; // Regex literal is a value
                    }
                    has_code_token = true;
                }
                Some(JsScope::Interpolation) | None => {
                    // Code Scope
                    // Check Comments
                    if c == '/' {
                        // Check next
                        if let Some((_, next_c)) = chars.peek() {
                            if *next_c == '/' {
                                // Line comment
                                break; // Stop processing line
                            } else if *next_c == '*' {
                                // Block comment
                                chars.next();
                                self.stack.push(JsScope::BlockComment);
                                continue;
                            }
                        }

                        // Not a comment.
                        // Check Regex vs Division
                        if !self.last_token_is_value {
                            // Start Regex
                            self.stack.push(JsScope::Regex { in_class: false });
                            has_code_token = true;
                            continue;
                        }
                        // Division operator
                        self.last_token_is_value = false; // Operator
                        has_code_token = true;
                    } else if c == '"' || c == '\'' || c == '`' {
                        self.stack.push(JsScope::String(c as u8));
                        has_code_token = true;
                    } else if c == '}' && matches!(self.stack.last(), Some(JsScope::Interpolation))
                    {
                        self.stack.pop();
                        self.last_token_is_value = true; // End of template block, back string?
                        // Wait, popping Interpolation brings us back to String(Backtick).
                        // BUT `process` loop matches `cloned()` scope.
                        // We just popped. Next iteration will match `String`.
                        // `last_token_is_value` doesn't apply inside String scope.
                        has_code_token = true;
                    } else {
                        // Regular code char
                        if !c.is_whitespace() {
                            has_code_token = true;
                            // Update value tracking
                            self.update_value_tracking(c);
                        }
                    }
                }
            }
        }

        usize::from(has_code_token)
    }

    fn update_value_tracking(&mut self, c: char) {
        if c.is_alphanumeric() || c == '_' || c == '$' || c == ')' || c == ']' || c == '}' {
            self.last_token_is_value = true;
        } else if "+-*%=<>&|!^~?:,;([{".contains(c) {
            self.last_token_is_value = false;
        }
        // '.' is handled? Usually implies property access? `foo.bar`.
        // `bar` is ident. `foo` is ident.
        // `.` is operator?
        // If I put `.` in op list?
        // `x.` ?
        // if `last_token_is_value` is false, `/` is regex.
        // `x. /regex/` -> invalid syntax usually (`x./regex/`).
        // `x.match(/regex/)`. `match` is ident. `(` is op. `/` is regex.
        // `x. /` invalid.
        // So `.` handling typically doesn't matter for Regex start detection.
    }

    pub fn reset(&mut self) {
        self.stack.clear();
        self.last_token_is_value = false;
        self.line_count = 0;
    }
}

// ============================================================================
// StatefulProcessor implementation
// ============================================================================

use super::super::processor_trait::StatefulProcessor;

/// State for `JavaScriptProcessor`.
#[derive(Debug, Clone, Default)]
pub struct JavaScriptState {
    /// Current scope stack (strings, comments, interpolations, regex).
    pub stack: Vec<JsScope>,
    /// Whether the last token was value-like (for regex heuristics).
    pub last_token_is_value: bool,
    /// Number of lines processed (for shebang detection).
    pub line_count: usize,
}

impl StatefulProcessor for JavaScriptProcessor {
    type State = JavaScriptState;

    fn get_state(&self) -> Self::State {
        JavaScriptState {
            stack: self.stack.clone(),
            last_token_is_value: self.last_token_is_value,
            line_count: self.line_count,
        }
    }

    fn set_state(&mut self, state: Self::State) {
        self.stack = state.stack;
        self.last_token_is_value = state.last_token_is_value;
        self.line_count = state.line_count;
    }

    fn is_in_multiline_context(&self) -> bool {
        // In a block comment, template literal, or other multi-line construct
        !self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_simple() {
        let mut p = JavaScriptProcessor::new();
        assert_eq!(p.process("let x = 1;"), 1);
        assert_eq!(p.process("// comment"), 0);
    }

    #[test]
    fn test_js_block_comment() {
        let mut p = JavaScriptProcessor::new();
        assert_eq!(p.process("/* start"), 0);
        assert_eq!(p.process("End */"), 0);
        assert_eq!(p.process("x = 1"), 1);
    }

    #[test]
    fn test_js_template_literal() {
        let mut p = JavaScriptProcessor::new();
        assert_eq!(p.process("x = `val ${ 1 }`"), 1);
        assert_eq!(p.process("y = `multi"), 1);
        assert_eq!(p.process("line`"), 1);
    }

    #[test]
    fn test_js_template_interpolation_comment() {
        let mut p = JavaScriptProcessor::new();
        assert_eq!(p.process("x = `start ${"), 1);
        assert_eq!(p.process("  // comment"), 0); // Should be 0
        assert_eq!(p.process("  y"), 1); // Code
        assert_eq!(p.process("}`"), 1);
    }

    #[test]
    fn test_js_regex() {
        let mut p = JavaScriptProcessor::new();
        assert_eq!(p.process("x = /regex/"), 1);
        assert_eq!(p.process("return /regex/"), 1);
    }
}
