// crates/infra/src/measurement/strategies/sloc_counter/processors/c_style.rs
//! C系言語のコメント処理
//!
//! C/C++/Java/JavaScript/Rust/Go/Kotlin等の
//! `//` 行コメントと `/* */` ブロックコメントを処理します。
//!
//! Note: Swift は拡張デリミタ文字列対応のため swift_style.rs に分離
//! Note: D言語は /+ +/ ネストコメント対応のため dlang_style.rs に分離

use super::super::string_utils::{find_outside_string_with_options, StringSkipOptions};

// ============================================================================
// CStyleProcessor: ネスト非対応 (C, C++, Java, Go, etc.)
// ============================================================================

/// C系言語プロセッサ (//, /* */) - ネスト非対応
pub struct CStyleProcessor {
    options: StringSkipOptions,
    in_block_comment: bool,
}

impl CStyleProcessor {
    pub fn new(options: StringSkipOptions) -> Self {
        Self {
            options,
            in_block_comment: false,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        if self.in_block_comment {
            // ブロックコメント内
            if let Some(pos) = line.find("*/") {
                self.in_block_comment = false;
                // 閉じた後にコードがあるかチェック
                let rest = &line[pos + 2..];
                if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                    return 1;
                }
            }
            return 0;
        }

        // 行コメント（文字列外）のみの行かチェック
        if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", &self.options) {
            // // より前にコードがあるか
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                // コメントのみの行
                return 0;
            }
            // コメント前にコードがある
            return 1;
        }

        // ブロックコメント開始をチェック（文字列外）
        if let Some(block_start) = find_outside_string_with_options(line, "/*", &self.options) {
            // /* より前にコードがあるか
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            
            // ブロックコメントが同じ行で閉じるか
            if let Some(block_end) = line[block_start + 2..].find("*/") {
                let after = &line[block_start + 2 + block_end + 2..];
                let has_code_after = !after.trim().is_empty() 
                    && find_outside_string_with_options(after, "//", &self.options).is_none_or(|p| p > 0);
                if has_code_before || has_code_after {
                    return 1;
                }
                return 0;
            } else {
                self.in_block_comment = true;
                if has_code_before {
                    return 1;
                }
            }
            return 0;
        }

        // コードがある行
        1
    }

    /// ブロックコメント内かどうか（テスト用）
    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment
    }
}

// ============================================================================
// NestingCStyleProcessor: ネスト対応 (Rust, Kotlin, Scala)
// ============================================================================

/// C系言語プロセッサ - ネスト対応 (Rust, Kotlin, Scala)
pub struct NestingCStyleProcessor {
    options: StringSkipOptions,
    in_block_comment: bool,
    block_comment_depth: usize,
}

impl NestingCStyleProcessor {
    pub fn new(options: StringSkipOptions) -> Self {
        Self {
            options,
            in_block_comment: false,
            block_comment_depth: 0,
        }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        let mut count = 0;
        self.process_internal(line, &mut count);
        count
    }

    fn process_internal(&mut self, line: &str, count: &mut usize) {
        // ネストされたブロックコメント内
        if self.block_comment_depth > 0 {
            self.process_nesting_block_line(line, count);
            return;
        }

        // 行コメント（文字列外）のみの行かチェック
        if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", &self.options) {
            let before = &line[..line_comment_pos];
            if before.trim().is_empty() {
                return;
            }
            *count += 1;
            return;
        }

        // ブロックコメント開始をチェック（文字列外）
        if let Some(block_start) = find_outside_string_with_options(line, "/*", &self.options) {
            let before = &line[..block_start];
            let has_code_before = !before.trim().is_empty();
            
            // ブロックコメント開始後の部分を処理
            self.block_comment_depth = 1;
            let rest = &line[block_start + 2..];
            self.process_nesting_block_line(rest, count);
            
            if has_code_before {
                *count += 1;
            }
            return;
        }

        // コードがある行
        *count += 1;
    }

    fn process_nesting_block_line(&mut self, line: &str, count: &mut usize) {
        let bytes = line.as_bytes();
        let mut i = 0;
        
        while i < bytes.len() {
            if i + 1 < bytes.len() {
                // /* を見つけたらネスト深度を増やす
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    self.block_comment_depth += 1;
                    i += 2;
                    continue;
                }
                // */ を見つけたらネスト深度を減らす
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    self.block_comment_depth -= 1;
                    i += 2;
                    
                    // 全てのコメントが閉じた
                    if self.block_comment_depth == 0 {
                        self.in_block_comment = false;
                        let rest = &line[i..];
                        if !rest.trim().is_empty() {
                            // 残りの部分を再帰的に処理
                            self.process_internal(rest, count);
                        }
                        return;
                    }
                    continue;
                }
            }
            i += 1;
        }
        
        // in_block_comment フラグも同期
        self.in_block_comment = self.block_comment_depth > 0;
    }

    /// ブロックコメント内かどうか（テスト用）
    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.in_block_comment || self.block_comment_depth > 0
    }
}

// ============================================================================
// 後方互換性のための関数 (既存テストのため維持)
// ============================================================================

/// C系スタイル (// と /* */) の処理 - StringSkipOptions対応版 (後方互換)
pub fn process_c_style_with_options(
    line: &str,
    options: &StringSkipOptions,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    // 内部的に CStyleProcessor を使用
    struct TempProcessor<'a> {
        options: &'a StringSkipOptions,
        in_block_comment: &'a mut bool,
    }
    
    let processor = TempProcessor { options, in_block_comment };
    
    if *processor.in_block_comment {
        // ブロックコメント内
        if let Some(pos) = line.find("*/") {
            *processor.in_block_comment = false;
            // 閉じた後にコードがあるかチェック
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                *count += 1;
            }
        }
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", processor.options) {
        // // より前にコードがあるか
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            // コメントのみの行
            return;
        }
        // コメント前にコードがある
        *count += 1;
        return;
    }

    // ブロックコメント開始をチェック（文字列外）
    if let Some(block_start) = find_outside_string_with_options(line, "/*", processor.options) {
        // /* より前にコードがあるか
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメントが同じ行で閉じるか
        if let Some(block_end) = line[block_start + 2..].find("*/") {
            let after = &line[block_start + 2 + block_end + 2..];
            let has_code_after = !after.trim().is_empty() 
                && find_outside_string_with_options(after, "//", processor.options).is_none_or(|p| p > 0);
            if has_code_before || has_code_after {
                *count += 1;
            }
        } else {
            *processor.in_block_comment = true;
            if has_code_before {
                *count += 1;
            }
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// ネストコメント対応 C系スタイル処理 - StringSkipOptions対応版 (後方互換)
pub fn process_nesting_c_style_with_options(
    line: &str,
    options: &StringSkipOptions,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    // ネストされたブロックコメント内
    if *block_comment_depth > 0 {
        process_nesting_block_comment_line_with_options(
            line,
            options,
            block_comment_depth,
            in_block_comment,
            count,
        );
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", options) {
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            return;
        }
        *count += 1;
        return;
    }

    // ブロックコメント開始をチェック（文字列外）
    if let Some(block_start) = find_outside_string_with_options(line, "/*", options) {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメント開始後の部分を処理
        *block_comment_depth = 1;
        let rest = &line[block_start + 2..];
        process_nesting_block_comment_line_with_options(
            rest,
            options,
            block_comment_depth,
            in_block_comment,
            count,
        );
        
        if has_code_before {
            *count += 1;
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// ネストされたブロックコメント行を処理 - StringSkipOptions対応版
fn process_nesting_block_comment_line_with_options(
    line: &str,
    options: &StringSkipOptions,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        if i + 1 < bytes.len() {
            // /* を見つけたらネスト深度を増やす
            if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                *block_comment_depth += 1;
                i += 2;
                continue;
            }
            // */ を見つけたらネスト深度を減らす
            if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                *block_comment_depth -= 1;
                i += 2;
                
                // 全てのコメントが閉じた
                if *block_comment_depth == 0 {
                    let rest = &line[i..];
                    if !rest.trim().is_empty() {
                        // 残りの部分を再帰的に処理
                        process_nesting_c_style_with_options(
                            rest,
                            options,
                            block_comment_depth,
                            in_block_comment,
                            count,
                        );
                    }
                    return;
                }
                continue;
            }
        }
        i += 1;
    }
    
    // in_block_comment フラグも同期
    *in_block_comment = *block_comment_depth > 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== CStyleProcessor テスト ====================

    #[test]
    fn test_c_style_processor_line_comment() {
        let mut p = CStyleProcessor::new(StringSkipOptions::c());
        assert_eq!(p.process("// comment"), 0);
        assert_eq!(p.process("int x = 1;"), 1);
    }

    #[test]
    fn test_c_style_processor_block_comment() {
        let mut p = CStyleProcessor::new(StringSkipOptions::c());
        assert_eq!(p.process("/* start"), 0);
        assert!(p.is_in_block_comment());
        assert_eq!(p.process("middle"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("code();"), 1);
    }

    #[test]
    fn test_c_style_processor_inline_comment() {
        let mut p = CStyleProcessor::new(StringSkipOptions::c());
        assert_eq!(p.process("int x = 1; // comment"), 1);
    }

    // ==================== NestingCStyleProcessor テスト ====================

    #[test]
    fn test_nesting_processor_nested_comment() {
        let mut p = NestingCStyleProcessor::new(StringSkipOptions::rust());
        assert_eq!(p.process("/* outer"), 0);
        assert_eq!(p.process("/* inner */"), 0);
        assert_eq!(p.process("still comment"), 0);
        assert_eq!(p.process("*/"), 0);
        assert!(!p.is_in_block_comment());
        assert_eq!(p.process("let x = 1;"), 1);
    }

    #[test]
    fn test_nesting_processor_single_line() {
        let mut p = NestingCStyleProcessor::new(StringSkipOptions::rust());
        assert_eq!(p.process("/* /* nested */ */ code();"), 1);
    }

    // ==================== 後方互換関数テスト ====================

    #[test]
    fn test_c_style_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_c_style_with_options("// comment", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_c_style_code_with_inline_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_c_style_with_options("int x = 1; // comment", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_c_style_block_comment_single_line() {
        let mut in_block = false;
        let mut count = 0;
        process_c_style_with_options("/* block comment */", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert_eq!(count, 0);
        assert!(!in_block);
    }

    #[test]
    fn test_c_style_block_comment_multiline() {
        let mut in_block = false;
        let mut count = 0;

        process_c_style_with_options("/* start", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_c_style_with_options("middle", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert!(in_block);
        assert_eq!(count, 0);

        process_c_style_with_options("*/", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert!(!in_block);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_c_style_code_with_block_comment() {
        let mut in_block = false;
        let mut count = 0;
        process_c_style_with_options("int x = 1; /* comment */", &StringSkipOptions::c(), &mut in_block, &mut count);
        assert_eq!(count, 1);
        assert!(!in_block);
    }

    #[test]
    fn test_c_no_nested_comments() {
        // C言語はネストコメント非対応
        let mut in_block = false;
        let mut count = 0;
        process_c_style_with_options("/* outer /* inner */ code_here(); */", &StringSkipOptions::c(), &mut in_block, &mut count);
        // C言語では最初の */ でコメント終了、code_here(); がコード扱い
        assert_eq!(count, 1);
    }

    // ==================== C++ Raw String リテラルテスト ====================

    #[test]
    fn test_cpp_raw_string_basic() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::cpp();
        process_c_style_with_options(r#"const char* s = R"(/* not a comment */)";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("int x = 1;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_cpp_raw_string_with_delimiter() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::cpp();
        process_c_style_with_options(r#"const char* s = R"foo(/* not a comment */)foo";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("int y = 2;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_cpp_raw_string_with_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::cpp();
        process_c_style_with_options(r#"const char* s = R"(// not a comment)";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("int z = 3;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Rust ネストコメントテスト ====================

    #[test]
    fn test_rust_nested_block_comment() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::rust();

        process_nesting_c_style_with_options("/* outer", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("  /* inner */", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("  still in comment", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("*/", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("let x = 1;", &options, &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_rust_nested_block_comment_single_line() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::rust();

        process_nesting_c_style_with_options("/* /* nested */ still comment */ code();", &options, &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_rust_nested_block_comment_deep() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::rust();

        process_nesting_c_style_with_options("/* level 1", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("/* level 2", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("/* level 3 */", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("back to level 2 */", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("back to level 1 */", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("code();", &options, &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_rust_code_before_nested_comment() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::rust();

        process_nesting_c_style_with_options("let x = 1; /* comment", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("  /* nested */", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("*/", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("let y = 2;", &options, &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Kotlin テスト ====================

    #[test]
    fn test_kotlin_nested_comments() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::java_kotlin();

        process_nesting_c_style_with_options("/* outer /* inner */ still comment */", &options, &mut block_depth, &mut in_block, &mut count);
        process_nesting_c_style_with_options("val x = 1", &options, &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_kotlin_text_block_with_block_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::java_kotlin();

        process_c_style_with_options(r#"val s = """/* not a comment */""";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("val z = 3;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Java テスト ====================

    #[test]
    fn test_java_no_nested_comments() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::java_kotlin();

        process_c_style_with_options("/* comment */", &options, &mut in_block, &mut count);
        process_c_style_with_options("int x = 1;", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_java_text_block_basic() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::java_kotlin();

        process_c_style_with_options(r#"String s = """text block""";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("int x = 1;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_java_text_block_with_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::java_kotlin();

        process_c_style_with_options(r#"String s = """// not a comment""";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("int y = 2;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== C# テスト ====================

    #[test]
    fn test_csharp_verbatim_string_basic() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::csharp();

        process_c_style_with_options(r#"var path = @"C:\MyFolder\file.txt";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("var x = 1;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_csharp_verbatim_string_with_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::csharp();

        process_c_style_with_options(r#"var regex = @"^# not a comment$";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("var y = 2;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_csharp_verbatim_string_escaped_quote() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::csharp();

        process_c_style_with_options(r#"var s = @"Quotes""Here""";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("var z = 3;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_csharp_verbatim_string_with_block_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::csharp();

        process_c_style_with_options(r#"var s = @"/* not a comment */";"#, &options, &mut in_block, &mut count);
        process_c_style_with_options("var w = 4;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Go テスト ====================

    #[test]
    fn test_go_raw_string_with_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::go();

        process_c_style_with_options("s := `/* not a comment */`", &options, &mut in_block, &mut count);
        process_c_style_with_options("x := 1", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== JavaScript/TypeScript テスト ====================

    #[test]
    fn test_js_template_literal_with_comment_marker() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::javascript();

        process_c_style_with_options("const s = `// not a comment`;", &options, &mut in_block, &mut count);
        process_c_style_with_options("const x = 1;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_ts_template_literal_with_block_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::javascript();

        process_c_style_with_options("const t = `/* still not a comment */`;", &options, &mut in_block, &mut count);
        process_c_style_with_options("const y = 2;", &options, &mut in_block, &mut count);
        assert_eq!(count, 2);
    }

    // ==================== Solidity テスト ====================

    #[test]
    fn test_solidity_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("// SPDX-License-Identifier: MIT", &options, &mut in_block, &mut count);
        process_c_style_with_options("pragma solidity ^0.8.0;", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_solidity_contract() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("/* ERC20 Token */", &options, &mut in_block, &mut count);
        process_c_style_with_options("contract Token {", &options, &mut in_block, &mut count);
        process_c_style_with_options("    uint256 public totalSupply; // total supply", &options, &mut in_block, &mut count);
        process_c_style_with_options("}", &options, &mut in_block, &mut count);
        assert_eq!(count, 3);
    }

    // ==================== Protocol Buffers テスト ====================

    #[test]
    fn test_protobuf_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("// Protocol buffer definition", &options, &mut in_block, &mut count);
        process_c_style_with_options(r#"syntax = "proto3";"#, &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_protobuf_message() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("message Person {", &options, &mut in_block, &mut count);
        process_c_style_with_options("  // Name field", &options, &mut in_block, &mut count);
        process_c_style_with_options("  string name = 1;", &options, &mut in_block, &mut count);
        process_c_style_with_options("  /* age field */", &options, &mut in_block, &mut count);
        process_c_style_with_options("  int32 age = 2;", &options, &mut in_block, &mut count);
        process_c_style_with_options("}", &options, &mut in_block, &mut count);
        assert_eq!(count, 4);
    }

    // ==================== Thrift テスト ====================

    #[test]
    fn test_thrift_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("// Thrift IDL", &options, &mut in_block, &mut count);
        process_c_style_with_options("namespace java com.example", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_thrift_struct() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("struct User {", &options, &mut in_block, &mut count);
        process_c_style_with_options("  /* user id */", &options, &mut in_block, &mut count);
        process_c_style_with_options("  1: i64 id,", &options, &mut in_block, &mut count);
        process_c_style_with_options("  2: string name, // user name", &options, &mut in_block, &mut count);
        process_c_style_with_options("}", &options, &mut in_block, &mut count);
        assert_eq!(count, 4);
    }

    // ==================== Verilog/SystemVerilog テスト ====================

    #[test]
    fn test_verilog_line_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("// SystemVerilog comment", &options, &mut in_block, &mut count);
        process_c_style_with_options("wire clk;", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_verilog_block_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("/* block comment */", &options, &mut in_block, &mut count);
        process_c_style_with_options("reg [7:0] data;", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_systemverilog_header() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("// Header file", &options, &mut in_block, &mut count);
        process_c_style_with_options("`define WIDTH 8", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    // ==================== リンカスクリプトテスト ====================

    #[test]
    fn test_linker_script_comment() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("/* Linker script */", &options, &mut in_block, &mut count);
        process_c_style_with_options("ENTRY(_start)", &options, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_linker_script_multiline() {
        let mut in_block = false;
        let mut count = 0;
        let options = StringSkipOptions::c();

        process_c_style_with_options("/*", &options, &mut in_block, &mut count);
        process_c_style_with_options(" * Memory layout", &options, &mut in_block, &mut count);
        process_c_style_with_options(" */", &options, &mut in_block, &mut count);
        process_c_style_with_options("MEMORY {", &options, &mut in_block, &mut count);
        process_c_style_with_options("    ROM : ORIGIN = 0x0, LENGTH = 64K", &options, &mut in_block, &mut count);
        process_c_style_with_options("}", &options, &mut in_block, &mut count);
        assert_eq!(count, 3);
    }
}
