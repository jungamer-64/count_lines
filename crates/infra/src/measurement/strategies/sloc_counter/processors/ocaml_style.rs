// crates/infra/src/measurement/strategies/sloc_counter/processors/ocaml_style.rs
//! OCaml/F#/Pascal系言語のコメント処理
//!
//! 対応するコメント構文:
//! - ブロックコメント: `(* ... *)` (ネスト対応)
//! - F# は // 行コメントも持つが、ここでは (* *) のみを処理

use super::super::string_utils::find_outside_string;

// ============================================================================
// OCamlProcessor 構造体 (新設計)
// ============================================================================

/// OCaml/F#/Pascal プロセッサ
///
/// - ブロックコメント: `(* *)` (ネスト対応)
/// - F# 行コメント: `//`
pub struct OCamlProcessor {
    block_depth: usize,
}

impl OCamlProcessor {
    pub fn new() -> Self {
        Self { block_depth: 0 }
    }

    /// 行を処理し、SLOCカウント (0 or 1) を返す
    pub fn process(&mut self, line: &str) -> usize {
        let trimmed = line.trim();

        // ブロックコメント内
        if self.block_depth > 0 {
            self.check_nesting(line);
            return 0;
        }

        // 空行
        if trimmed.is_empty() {
            return 0;
        }

        // F# // 行コメント
        if let Some(pos) = find_outside_string(line, "//") {
            let before = &line[..pos];
            return if !before.trim().is_empty() { 1 } else { 0 };
        }

        // ブロックコメント開始判定
        if let Some(pos) = find_outside_string(line, "(*") {
            let before = &line[..pos];
            let has_code_before = !before.trim().is_empty();

            self.block_depth = 1;
            let rest = &line[pos + 2..];
            self.check_nesting(rest);

            return if has_code_before { 1 } else { 0 };
        }

        1
    }

    fn check_nesting(&mut self, content: &str) {
        let bytes = content.as_bytes();
        let mut i = 0;

        while i < bytes.len() {
            if i + 1 < bytes.len() && bytes[i] == b'(' && bytes[i + 1] == b'*' {
                self.block_depth += 1;
                i += 2;
                continue;
            }

            if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b')' {
                self.block_depth = self.block_depth.saturating_sub(1);
                i += 2;
                continue;
            }

            i += 1;
        }
    }

    #[cfg(test)]
    pub fn is_in_block_comment(&self) -> bool {
        self.block_depth > 0
    }
}

impl Default for OCamlProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 後方互換性のための関数 (レガシー)
// ============================================================================

/// OCaml スタイル ((* *)) の処理
/// 
/// # Arguments
/// * `line` - 処理する行
/// * `in_block_comment` - ブロックコメント内かどうか
/// * `block_comment_depth` - ブロックコメントのネスト深度
/// * `count` - SLOCカウント
pub fn process_ocaml_style(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
    count: &mut usize,
) {
    let trimmed = line.trim();
    
    // ブロックコメント内の処理
    if *in_block_comment {
        process_ocaml_block_comment(line, in_block_comment, block_comment_depth);
        return;
    }
    
    // 空行
    if trimmed.is_empty() {
        return;
    }
    
    // F#/OCaml の // 行コメント対応（オプション）
    // 注: OCaml は // をサポートしないが、F# はサポートする
    // ここでは両方に対応するため // もチェック
    if let Some(pos) = find_outside_string(line, "//") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }
    
    // ブロックコメント開始判定
    if let Some(pos) = find_outside_string(line, "(*") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        // ブロックコメント開始
        *in_block_comment = true;
        *block_comment_depth = 1;
        
        // 残りの部分でさらにネストや終了をチェック
        let rest = &line[pos + 2..];
        check_ocaml_block_nesting(rest, in_block_comment, block_comment_depth);
        return;
    }
    
    // コードとしてカウント
    *count += 1;
}

/// ブロックコメント内の処理（ネスト対応）
fn process_ocaml_block_comment(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
) {
    check_ocaml_block_nesting(line, in_block_comment, block_comment_depth);
}

/// OCaml ブロックコメントのネスト処理
fn check_ocaml_block_nesting(
    content: &str,
    in_block_comment: &mut bool,
    depth: &mut usize,
) {
    let bytes = content.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // ネストされた (* の開始
        if i + 1 < bytes.len() && bytes[i] == b'(' && bytes[i + 1] == b'*' {
            *depth += 1;
            i += 2;
            continue;
        }
        
        // *) の終了
        if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b')' {
            *depth = depth.saturating_sub(1);
            if *depth == 0 {
                *in_block_comment = false;
            }
            i += 2;
            continue;
        }
        
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocaml_block_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_ocaml_style("(* comment *)", &mut in_block, &mut depth, &mut count);
        process_ocaml_style("let x = 1", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_ocaml_nested_block_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_ocaml_style("(* outer (* inner *) still outer *)", &mut in_block, &mut depth, &mut count);
        process_ocaml_style("let y = 2", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
        assert!(!in_block);
    }

    #[test]
    fn test_ocaml_multiline_block_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_ocaml_style("(*", &mut in_block, &mut depth, &mut count);
        assert!(in_block);
        process_ocaml_style("  multiline", &mut in_block, &mut depth, &mut count);
        process_ocaml_style("*)", &mut in_block, &mut depth, &mut count);
        assert!(!in_block);
        process_ocaml_style("let z = 3", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_fsharp_line_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_ocaml_style("// F# comment", &mut in_block, &mut depth, &mut count);
        process_ocaml_style("let a = 1", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_pascal_block_comment() {
        let mut in_block = false;
        let mut depth = 0;
        let mut count = 0;

        process_ocaml_style("(* Pascal comment *)", &mut in_block, &mut depth, &mut count);
        process_ocaml_style("var x: Integer;", &mut in_block, &mut depth, &mut count);
        assert_eq!(count, 1);
    }
}
