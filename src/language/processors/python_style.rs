// src/language/processors/python_style.rs
//! Python言語のコメント処理
//!
//! Python固有の対応:
//! - Docstring: `"""..."""` / `'''...'''`
//! - f-string: `f"..."`, `F"..."` 等の文字列プレフィックス
//! - 複合プレフィックス: `fr"..."`, `rf"..."` 等
//! - shebang行の除外

use super::super::processor_trait::LineProcessor;

#[derive(Debug, Clone, PartialEq, Eq)]
enum PythonScope {
    Interpolation, // { ... }
    String(PythonStringState),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
struct PythonStringState {
    quote: u8,    // " or '
    triple: bool, // """ or '''
    is_f_string: bool,
    is_raw: bool,
    is_doc_comment: bool, // Treat content as comment?
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
    pub fn new() -> Self {
        Self::default()
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    #[allow(clippy::too_many_lines)]
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // shebang行を除外 (最初の行のみ)
        if trimmed.starts_with("#!") && self.line_count == 0 {
            self.line_count += 1;
            return 0;
        }
        self.line_count += 1;

        let mut has_code_token = false;

        // Stack state determines initial has_code_token
        if let Some(PythonScope::String(state)) = self.stack.last()
            && !state.is_doc_comment
        {
            has_code_token = true; // Continuation of data string
        }

        let mut chars = line.char_indices().peekable();

        // Use `_` for index as it is unused currently (we scan sequentially)
        while let Some((_, c)) = chars.next() {
            // Handle active string state
            if let Some(PythonScope::String(state)) = self.stack.last().cloned() {
                // Check end
                let quote_char = state.quote as char;
                if c == quote_char {
                    if state.triple {
                        // Check next 2 chars
                        let mut success = false;
                        if let Some((_, c2)) = chars.peek()
                            && *c2 == quote_char
                        {
                            // Match 2nd. Consume.
                            chars.next();
                            if let Some((_, c3)) = chars.peek()
                                && *c3 == quote_char
                            {
                                // Match 3rd. Consume.
                                chars.next();
                                success = true;
                            }
                        }

                        if success {
                            self.stack.pop();
                            if !state.is_doc_comment {
                                has_code_token = true;
                            } // closing quote is code
                            continue;
                        }

                        // Not a triple quote end. Treat as regular chars.
                        if !state.is_doc_comment && !c.is_whitespace() {
                            has_code_token = true;
                        }
                    } else {
                        // Single quote end
                        self.stack.pop();
                        if !state.is_doc_comment {
                            has_code_token = true;
                        }
                        continue;
                    }
                }

                // Escape handling
                if c == '\\' {
                    if state.is_raw {
                        // Raw string: backslash is literal, but escapes quote?
                        if chars
                            .peek()
                            .is_some_and(|&(_, next_c)| next_c == quote_char)
                        {
                            chars.next(); // Consume quote as escaped (part of string)
                        }
                        // Else: backslash is just a char
                    } else {
                        // Normal string: escape next char
                        chars.next();
                    }
                    if !state.is_doc_comment {
                        has_code_token = true;
                    }
                    continue;
                }

                // Interpolation handling (f-string)
                if state.is_f_string && c == '{' {
                    // check for {{ (escape)
                    let mut is_escape = false;
                    if let Some((_, next_c)) = chars.peek()
                        && *next_c == '{'
                    {
                        chars.next();
                        is_escape = true;
                    }
                    if !is_escape {
                        self.stack.push(PythonScope::Interpolation);
                        // { token is code structure
                        has_code_token = true;
                    }
                }

                if !state.is_doc_comment && !c.is_whitespace() {
                    has_code_token = true;
                }
            } else {
                // Code / Interpolation scope

                // Check comment
                if c == '#' {
                    break; // Ignore rest of line
                }

                // Check string start
                // Prefixes: f, r, u, b
                let mut is_prefix = false;
                let lower_c = c.to_ascii_lowercase();
                if lower_c == 'f' || lower_c == 'r' || lower_c == 'u' || lower_c == 'b' {
                    is_prefix = true;
                }

                if c == '"' || c == '\'' || is_prefix {
                    // Potential string start.
                    let mut is_start = false;
                    let mut quote = 0u8;
                    let mut triple = false;
                    let mut raw = false;
                    let mut f_string = false;

                    // State for prefix
                    let mut p_f = false;
                    let mut p_r = false;
                    let mut p_b = false;
                    let mut p_u = false;

                    let mut current = c;
                    let mut is_valid_seq = true;

                    loop {
                        let lc = current.to_ascii_lowercase();
                        if current == '"' || current == '\'' {
                            // Quote found! String starts.
                            quote = current as u8;
                            is_start = true;
                            break;
                        }

                        if lc == 'f' && !p_f {
                            p_f = true;
                            f_string = true;
                        } else if lc == 'r' && !p_r {
                            p_r = true;
                            raw = true;
                        } else if lc == 'b' && !p_b {
                            p_b = true;
                        } else if lc == 'u' && !p_u {
                            p_u = true;
                        } else {
                            is_valid_seq = false;
                            break;
                        }

                        // Next char
                        if let Some((_, next_c)) = chars.next() {
                            current = next_c;
                            // We consume prefixes.
                        } else {
                            is_valid_seq = false;
                            break;
                        }
                    }

                    if is_start && is_valid_seq {
                        // Check triple
                        if chars.peek().is_some_and(|&(_, c2)| c2 == quote as char) {
                            chars.next(); // consume 2nd
                            if let Some((_, c3)) = chars.peek() {
                                if *c3 == quote as char {
                                    chars.next(); // consume 3rd
                                    triple = true;
                                } else {
                                    // "" is valid empty string
                                    has_code_token = true;
                                    continue; // Done with string
                                }
                            } else {
                                has_code_token = true;
                                continue;
                            }
                        }

                        // Determine is_doc_comment
                        // Only if we haven't seen code tokens yet IN THIS LINE
                        let is_doc_comment = triple && !has_code_token;

                        self.stack.push(PythonScope::String(PythonStringState {
                            quote,
                            triple,
                            is_f_string: f_string,
                            is_raw: raw,
                            is_doc_comment,
                        }));

                        if !is_doc_comment {
                            has_code_token = true;
                        }
                        continue;
                    }

                    // If failed, treated as identifiers/code.
                    has_code_token = true;
                } else {
                    // Not string start.
                    if !c.is_whitespace() {
                        has_code_token = true;
                    }
                    // Check closing } for interpolation
                    if c == '}' && matches!(self.stack.last(), Some(PythonScope::Interpolation)) {
                        self.stack.pop();
                    }
                }
            }
        }

        usize::from(has_code_token)
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
}
