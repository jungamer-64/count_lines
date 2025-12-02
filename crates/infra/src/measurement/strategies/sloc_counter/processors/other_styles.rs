// crates/infra/src/measurement/strategies/sloc_counter/processors/other_styles.rs
//! その他の言語のコメント処理
//!
//! Lua, HTML/XML, SQL, Haskell, Lisp, Erlang, Fortran, MATLAB等を処理します。

use super::super::string_utils::find_outside_string_sql;

/// Lua スタイル (-- と --[=*[ ]=*]) の処理
///
/// Lua のブロックコメントは `--[[` で始まり `]]` で終わる。
/// 等号を任意の数だけ挟める: `--[=[`, `--[==[`, etc.
/// 対応する閉じ括弧も同じ数の等号が必要: `]=]`, `]==]`, etc.
pub fn process_lua_style(
    line: &str,
    in_block_comment: &mut bool,
    lua_block_level: &mut usize,
    count: &mut usize,
) {
    if *in_block_comment {
        // ブロックコメント内: 対応する閉じ括弧を探す
        if let Some(_) = find_lua_block_end(line, *lua_block_level) {
            *in_block_comment = false;
            *lua_block_level = 0;
        }
        return;
    }

    // 行コメント
    if line.starts_with("--") {
        // ブロックコメント開始かチェック: --[[, --[=[, --[==[, etc.
        if let Some(level) = check_lua_block_start(&line[2..]) {
            // ブロックコメント開始
            // 同じ行で閉じるかチェック
            let after_open = skip_lua_block_open(&line[2..]);
            if find_lua_block_end(after_open, level).is_some() {
                // 同じ行で閉じる = コメント行
                return;
            }
            *in_block_comment = true;
            *lua_block_level = level;
        }
        // 行コメントまたはブロックコメント開始 = SLOCではない
        return;
    }

    *count += 1;
}

/// Lua ブロックコメント開始をチェック
/// `[` で始まり、0個以上の `=` の後に `[` が続く場合、等号の数を返す
fn check_lua_block_start(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] != b'[' {
        return None;
    }

    let mut i = 1;
    let mut level = 0;

    // 等号をカウント
    while i < bytes.len() && bytes[i] == b'=' {
        level += 1;
        i += 1;
    }

    // 2番目の [ を確認
    if i < bytes.len() && bytes[i] == b'[' {
        return Some(level);
    }

    None
}

/// Lua ブロックコメント開始部分をスキップして残りを返す
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

/// Lua ブロックコメント終了を検索
/// `]` + level個の `=` + `]` を探す
fn find_lua_block_end(s: &str, level: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b']' {
            // 等号をカウント
            let mut eq_count = 0;
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == b'=' {
                eq_count += 1;
                j += 1;
            }
            // 閉じ括弧を確認
            if j < bytes.len() && bytes[j] == b']' && eq_count == level {
                return Some(j + 1);
            }
        }
        i += 1;
    }

    None
}

/// HTML スタイル (<!-- -->) の処理
pub fn process_html_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if line.contains("-->") {
            *in_block_comment = false;
            // --> の後にコードがあるかチェック
            if let Some(pos) = line.find("-->") {
                let rest = &line[pos + 3..];
                if !rest.trim().is_empty() {
                    *count += 1;
                }
            }
        }
        return;
    }

    if let Some(start) = line.find("<!--") {
        let before = &line[..start];
        let has_code_before = !before.trim().is_empty();

        if let Some(end_offset) = line[start + 4..].find("-->") {
            let after = &line[start + 4 + end_offset + 3..];
            if has_code_before || !after.trim().is_empty() {
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

    *count += 1;
}

/// SQL スタイル (-- と /* */) の処理
/// 
/// SQL の文字列リテラル ('...' と "...") 内のコメントマーカーは無視する
pub fn process_sql_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() {
                // 残りの部分を再帰的に処理
                process_sql_style(rest, in_block_comment, count);
            }
        }
        return;
    }

    // 行コメント (文字列外)
    if let Some(line_comment_pos) = find_outside_string_sql(line, "--") {
        // -- より前にコードがあるかチェック
        let before = &line[..line_comment_pos];
        
        // -- より前にブロックコメント開始があるかチェック
        if let Some(block_start) = find_outside_string_sql(before, "/*") {
            // ブロックコメントの方が先にある
            process_sql_block_comment(line, block_start, in_block_comment, count);
            return;
        }
        
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }

    // ブロックコメント開始 (文字列外)
    if let Some(block_start) = find_outside_string_sql(line, "/*") {
        process_sql_block_comment(line, block_start, in_block_comment, count);
        return;
    }

    *count += 1;
}

/// SQL ブロックコメント処理のヘルパー
fn process_sql_block_comment(
    line: &str,
    block_start: usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let before = &line[..block_start];
    let has_code_before = !before.trim().is_empty();

    let after_start = &line[block_start + 2..];
    if let Some(end_offset) = after_start.find("*/") {
        // 同じ行で閉じる
        let after = &after_start[end_offset + 2..];
        if has_code_before {
            *count += 1;
        } else if !after.trim().is_empty() {
            // コメント後の残りを再帰的に処理
            process_sql_style(after, in_block_comment, count);
        }
    } else {
        // 閉じられていない = ブロックコメント開始
        *in_block_comment = true;
        if has_code_before {
            *count += 1;
        }
    }
}

/// Haskell スタイル (-- と {- -}) の処理 - ネスト対応
///
/// Haskell のブロックコメント `{- -}` はネスト可能
pub fn process_haskell_style(
    line: &str,
    in_block_comment: &mut bool,
    block_comment_depth: &mut usize,
    count: &mut usize,
) {
    // ネストされたブロックコメント内
    if *block_comment_depth > 0 {
        process_nesting_haskell_block(line, block_comment_depth, in_block_comment, count);
        return;
    }

    // 行コメント
    if line.starts_with("--") {
        return;
    }

    // ブロックコメント開始 {-
    if let Some(block_start) = line.find("{-") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

        // ブロックコメント開始
        *block_comment_depth = 1;
        let rest = &line[block_start + 2..];
        process_nesting_haskell_block(rest, block_comment_depth, in_block_comment, count);

        if has_code_before {
            *count += 1;
        }
        return;
    }

    *count += 1;
}

/// ネストされた Haskell ブロックコメント行を処理
fn process_nesting_haskell_block(
    line: &str,
    block_comment_depth: &mut usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() {
            // {- を見つけたらネスト深度を増やす
            if bytes[i] == b'{' && bytes[i + 1] == b'-' {
                *block_comment_depth += 1;
                i += 2;
                continue;
            }
            // -} を見つけたらネスト深度を減らす
            if bytes[i] == b'-' && bytes[i + 1] == b'}' {
                *block_comment_depth -= 1;
                i += 2;

                // 全てのコメントが閉じた
                if *block_comment_depth == 0 {
                    let rest = &line[i..];
                    if !rest.trim().is_empty() {
                        // 残りの部分を再帰的に処理
                        process_haskell_style(rest, in_block_comment, block_comment_depth, count);
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

/// Lisp スタイル (;) の処理
pub fn process_lisp_style(line: &str, count: &mut usize) {
    if line.starts_with(';') {
        return;
    }
    *count += 1;
}

/// Erlang スタイル (%) の処理
pub fn process_erlang_style(line: &str, count: &mut usize) {
    if line.starts_with('%') {
        return;
    }
    *count += 1;
}

/// Fortran スタイル (!) の処理
pub fn process_fortran_style(line: &str, count: &mut usize) {
    // Fortran: ! で始まるコメント、または C/c/*/d/D で始まる固定形式コメント
    if line.starts_with('!')
        || line.starts_with('C')
        || line.starts_with('c')
        || line.starts_with('*')
    {
        return;
    }
    *count += 1;
}

/// MATLAB スタイル (% と %{ %}) の処理
pub fn process_matlab_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if line.trim() == "%}" {
            *in_block_comment = false;
        }
        return;
    }

    if line.trim() == "%{" {
        *in_block_comment = true;
        return;
    }

    if line.starts_with('%') {
        return;
    }

    *count += 1;
}

/// Batch スタイル (REM と ::) の処理
///
/// Windows バッチファイルのコメント:
/// - `REM` (大文字小文字不問) で始まる行
/// - `::` で始まる行 (ラベルの特殊用法としてのコメント)
pub fn process_batch_style(line: &str, count: &mut usize) {
    let trimmed = line.trim();
    
    // REM コメント (大文字小文字不問)
    // "REM" の後にスペースか行末が必要
    let upper = trimmed.to_uppercase();
    if upper == "REM" || upper.starts_with("REM ") || upper.starts_with("REM\t") {
        return;
    }
    
    // :: コメント (ラベルの特殊用法)
    if trimmed.starts_with("::") {
        return;
    }
    
    // @ プレフィックス付きの REM
    if trimmed.starts_with('@') {
        let after_at = trimmed[1..].trim_start();
        let upper_after = after_at.to_uppercase();
        if upper_after == "REM" || upper_after.starts_with("REM ") || upper_after.starts_with("REM\t") {
            return;
        }
    }
    
    *count += 1;
}

/// Assembly (NASM/MASM) スタイル (; のみ) の処理
///
/// Intel形式アセンブリ (NASM, MASM等):
/// - `;` 以降が行コメント
pub fn process_assembly_style(line: &str, count: &mut usize) {
    // ; から始まる場合はコメント行
    if line.starts_with(';') {
        return;
    }
    
    // 行中に ; がある場合、その前にコードがあればカウント
    if let Some(pos) = line.find(';') {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }
    
    *count += 1;
}

/// GAS (GNU Assembler) スタイル (# と /* */) の処理
///
/// AT&T形式アセンブリ (GAS, ARM等):
/// - `#` 以降が行コメント (プリプロセッサも)
/// - `/* */` ブロックコメント
/// - 一部のGASでは `//` も使用可能だが、`#` が主流
pub fn process_gas_assembly_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() {
                // 残りを再帰処理
                process_gas_assembly_style(rest, in_block_comment, count);
            }
        }
        return;
    }

    // # 行コメント
    if line.starts_with('#') || line.starts_with('@') {
        // @ も ARM GAS でコメント
        return;
    }
    
    // 行中の # コメント
    if let Some(hash_pos) = line.find('#') {
        let before = &line[..hash_pos];
        
        // # の前に /* があるかチェック
        if let Some(block_start) = before.find("/*") {
            // ブロックコメントが先
            process_gas_block_comment(line, block_start, in_block_comment, count);
            return;
        }
        
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }

    // /* ブロックコメント
    if let Some(block_start) = line.find("/*") {
        process_gas_block_comment(line, block_start, in_block_comment, count);
        return;
    }

    *count += 1;
}

/// GAS ブロックコメント処理のヘルパー
fn process_gas_block_comment(
    line: &str,
    block_start: usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let before = &line[..block_start];
    let has_code_before = !before.trim().is_empty();

    let after_start = &line[block_start + 2..];
    if let Some(end_offset) = after_start.find("*/") {
        // 同じ行で閉じる
        let after = &after_start[end_offset + 2..];
        if has_code_before {
            *count += 1;
        } else if !after.trim().is_empty() {
            process_gas_assembly_style(after, in_block_comment, count);
        }
    } else {
        *in_block_comment = true;
        if has_code_before {
            *count += 1;
        }
    }
}

/// VHDL スタイル (-- のみ) の処理
///
/// VHDL:
/// - `--` 以降が行コメント
/// - VHDL-2008ではブロックコメントがあるが、多くの処理系が未対応なので行コメントのみ
pub fn process_vhdl_style(line: &str, count: &mut usize) {
    // -- から始まる場合はコメント行
    if line.starts_with("--") {
        return;
    }
    
    // 行中に -- がある場合、その前にコードがあればカウント
    if let Some(pos) = line.find("--") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }
    
    *count += 1;
}

/// Visual Basic / VBA / VBScript スタイル (' と REM) の処理
///
/// VB系言語のコメント:
/// - `'` で始まる行コメント
/// - `REM` で始まる行コメント (大文字小文字不問)
/// - 行中の `'` 以降もコメント（文字列リテラル外）
pub fn process_visual_basic_style(line: &str, count: &mut usize) {
    let trimmed = line.trim();
    
    // ' で始まるコメント行
    if trimmed.starts_with('\'') {
        return;
    }
    
    // REM コメント (大文字小文字不問)
    let upper = trimmed.to_uppercase();
    if upper == "REM" || upper.starts_with("REM ") || upper.starts_with("REM\t") {
        return;
    }
    
    // 文字列リテラル外の ' を探す
    // VBの文字列は "" でエスケープ、\ はエスケープなし
    let bytes = line.as_bytes();
    let mut i = 0;
    let mut in_string = false;
    
    while i < bytes.len() {
        if in_string {
            if bytes[i] == b'"' {
                // "" はエスケープされた "
                if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                    i += 2;
                    continue;
                }
                in_string = false;
            }
            i += 1;
            continue;
        }
        
        if bytes[i] == b'"' {
            in_string = true;
            i += 1;
            continue;
        }
        
        if bytes[i] == b'\'' {
            // ' 以前にコードがあればカウント
            let before = &line[..i];
            if !before.trim().is_empty() {
                *count += 1;
            }
            return;
        }
        
        i += 1;
    }
    
    *count += 1;
}
