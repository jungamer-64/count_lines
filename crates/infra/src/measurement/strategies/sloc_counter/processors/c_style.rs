// crates/infra/src/measurement/strategies/sloc_counter/processors/c_style.rs
//! C系言語のコメント処理
//!
//! C/C++/Java/JavaScript/Rust/Go/Swift/Kotlin等の
//! `//` 行コメントと `/* */` ブロックコメントを処理します。

use super::super::string_utils::{
    find_outside_string, find_outside_string_swift,
    find_outside_string_with_options, StringSkipOptions,
};

/// C系スタイル (// と /* */) の処理 - StringSkipOptions対応版
/// 
/// 言語に応じたStringSkipOptionsを渡すことで、
/// 各言語固有の文字列リテラル構文を正しくスキップできます。
pub fn process_c_style_with_options(
    line: &str,
    options: &StringSkipOptions,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        // ブロックコメント内
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            // 閉じた後にコードがあるかチェック
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                *count += 1;
            }
        }
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string_with_options(line, "//", options) {
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
    if let Some(block_start) = find_outside_string_with_options(line, "/*", options) {
        // /* より前にコードがあるか
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメントが同じ行で閉じるか
        if let Some(block_end) = line[block_start + 2..].find("*/") {
            let after = &line[block_start + 2 + block_end + 2..];
            let has_code_after = !after.trim().is_empty() 
                && find_outside_string_with_options(after, "//", options).is_none_or(|p| p > 0);
            if has_code_before || has_code_after {
                *count += 1;
            }
        } else {
            *in_block_comment = true;
            if has_code_before {
                *count += 1;
            }
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// ネストコメント対応 C系スタイル処理 - StringSkipOptions対応版
///
/// 言語に応じたStringSkipOptionsを渡すことで、
/// 各言語固有の文字列リテラル構文を正しくスキップできます。
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

/// ネストされたブロックコメント行を処理
fn process_nesting_block_comment_line(
    line: &str,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    process_nesting_block_comment_line_with_options(
        line,
        &StringSkipOptions::rust(),
        block_comment_depth,
        in_block_comment,
        count,
    )
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

/// Swift スタイル処理（拡張デリミタ文字列 #"..."# と多重引用符 """...""" 対応）
pub fn process_swift_style(
    line: &str,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    // ネストされたブロックコメント内
    if *block_comment_depth > 0 {
        process_nesting_block_comment_line(line, block_comment_depth, in_block_comment, count);
        return;
    }

    // 行コメント（文字列外）のみの行かチェック - Swift文字列対応
    if let Some(line_comment_pos) = find_outside_string_swift(line, "//") {
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            return;
        }
        *count += 1;
        return;
    }

    // ブロックコメント開始をチェック（文字列外、Swift文字列対応）
    if let Some(block_start) = find_outside_string_swift(line, "/*") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        // ブロックコメント開始後の部分を処理
        *block_comment_depth = 1;
        let rest = &line[block_start + 2..];
        process_nesting_block_comment_line(rest, block_comment_depth, in_block_comment, count);
        
        if has_code_before {
            *count += 1;
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// D言語スタイル処理 (//, /* */, /+ +/ ネスト対応)
///
/// D言語は3種類のコメントをサポート:
/// - `//` 行コメント
/// - `/* */` ブロックコメント（ネスト不可）
/// - `/+ +/` ブロックコメント（ネスト可能）
pub fn process_dlang_style(
    line: &str,
    in_block_comment: &mut bool,
    in_nesting_block: &mut bool,
    nesting_block_depth: &mut usize,
    count: &mut usize,
) {
    // /+ +/ ネストブロックコメント内
    if *nesting_block_depth > 0 {
        process_dlang_nesting_block(line, in_nesting_block, nesting_block_depth, in_block_comment, count);
        return;
    }
    
    // 通常の /* */ ブロックコメント内
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("//") {
                // 残りを再帰処理
                process_dlang_style(rest, in_block_comment, in_nesting_block, nesting_block_depth, count);
            }
        }
        return;
    }

    // 行コメント（文字列外）のみの行かチェック
    if let Some(line_comment_pos) = find_outside_string(line, "//") {
        let before = &line[..line_comment_pos];
        if before.trim().is_empty() {
            return;
        }
        *count += 1;
        return;
    }

    // /+ ネストブロックコメント開始をチェック（文字列外）
    if let Some(nesting_start) = find_outside_string(line, "/+") {
        // /* より前に /+ があるかチェック
        let block_start = find_outside_string(line, "/*");
        
        if block_start.is_none() || nesting_start < block_start.unwrap() {
            // /+ が先
            let before = &line[..nesting_start];
            let has_code_before = !before.trim().is_empty();
            
            *nesting_block_depth = 1;
            *in_nesting_block = true;
            let rest = &line[nesting_start + 2..];
            process_dlang_nesting_block(rest, in_nesting_block, nesting_block_depth, in_block_comment, count);
            
            if has_code_before {
                *count += 1;
            }
            return;
        }
    }

    // /* ブロックコメント開始をチェック（文字列外）
    if let Some(block_start) = find_outside_string(line, "/*") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty();
        
        if let Some(block_end) = line[block_start + 2..].find("*/") {
            let after = &line[block_start + 2 + block_end + 2..];
            if has_code_before {
                *count += 1;
            } else if !after.trim().is_empty() {
                // 残りを再帰処理
                process_dlang_style(after, in_block_comment, in_nesting_block, nesting_block_depth, count);
            }
        } else {
            *in_block_comment = true;
            if has_code_before {
                *count += 1;
            }
        }
        return;
    }

    // コードがある行
    *count += 1;
}

/// D言語の /+ +/ ネストブロックコメント行を処理
fn process_dlang_nesting_block(
    line: &str,
    in_nesting_block: &mut bool,
    nesting_block_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        if i + 1 < bytes.len() {
            // /+ を見つけたらネスト深度を増やす
            if bytes[i] == b'/' && bytes[i + 1] == b'+' {
                *nesting_block_depth += 1;
                i += 2;
                continue;
            }
            // +/ を見つけたらネスト深度を減らす
            if bytes[i] == b'+' && bytes[i + 1] == b'/' {
                *nesting_block_depth -= 1;
                i += 2;
                
                // 全てのコメントが閉じた
                if *nesting_block_depth == 0 {
                    *in_nesting_block = false;
                    let rest = &line[i..];
                    if !rest.trim().is_empty() {
                        // 残りの部分を再帰的に処理
                        process_dlang_style(rest, in_block_comment, in_nesting_block, nesting_block_depth, count);
                    }
                    return;
                }
                continue;
            }
        }
        i += 1;
    }
    
    // まだコメント内
    *in_nesting_block = *nesting_block_depth > 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 基本 C スタイルテスト ====================

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

    // ==================== Swift テスト ====================

    #[test]
    fn test_swift_nested_comments() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("/* outer", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("  /* nested */", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("*/", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_swift_extended_delimiter_string() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r##"let s = #"/* not a comment */"#"##, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_swift_extended_delimiter_double_hash() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r###"let s = ##"contains "# but not end"##"###, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let y = 2", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_swift_multiline_string() {
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style(r#"let s = """/* not a comment */""""#, &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let z = 3", &mut block_depth, &mut in_block, &mut count);
        assert_eq!(count, 2);
        assert!(!in_block);
    }

    #[test]
    fn test_swift_hash_not_comment() {
        // Swift では # はコメント開始ではない（拡張デリミタの一部）
        let mut block_depth = 0;
        let mut in_block = false;
        let mut count = 0;

        process_swift_style("let hash = #selector(foo)", &mut block_depth, &mut in_block, &mut count);
        process_swift_style("let x = 1", &mut block_depth, &mut in_block, &mut count);
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

    // ==================== D 言語テスト ====================

    #[test]
    fn test_dlang_line_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("// comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int y = 2; // inline comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_dlang_block_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/* block comment */", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int z = 3;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_basic() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("  nesting comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int a = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_nested() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ outer", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ inner +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("still in outer +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int b = 2;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_single_line() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ /+ nested +/ still in outer +/ int c = 3;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_nesting_comment_deep() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/+ level 1", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ level 2", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ level 3 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("back to level 2 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("back to level 1 +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int d = 4;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_mixed_comments() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("/* block */ /+ nesting +/ int x = 1;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_dlang_code_before_nesting_comment() {
        let mut in_block = false;
        let mut in_nesting = false;
        let mut nesting_depth = 0;
        let mut count = 0;

        process_dlang_style("int x = 1; /+ comment", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("/+ nested +/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("+/", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
        process_dlang_style("int y = 2;", &mut in_block, &mut in_nesting, &mut nesting_depth, &mut count);
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
