// crates/infra/src/measurement/strategies/sloc_counter/processors/other_styles.rs
//! その他の言語のコメント処理
//!
//! Lua, HTML/XML, SQL, Haskell, Lisp, Erlang, Fortran, MATLAB等を処理します。

/// Lua スタイル (-- と --[[ ]]) の処理
pub fn process_lua_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if line.contains("]]") {
            *in_block_comment = false;
        }
        return;
    }

    if line.starts_with("--[[") || line.starts_with("--[=[") {
        *in_block_comment = true;
        return;
    }

    if line.starts_with("--") {
        return;
    }

    *count += 1;
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
pub fn process_sql_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("--") {
                *count += 1;
            }
        }
        return;
    }

    // ブロックコメント開始
    if let Some(block_start) = line.find("/*") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

        if let Some(end_offset) = line[block_start + 2..].find("*/") {
            let after = &line[block_start + 2 + end_offset + 2..];
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

    // 行コメント
    if line.starts_with("--") {
        return;
    }

    *count += 1;
}

/// Haskell スタイル (-- と {- -}) の処理
pub fn process_haskell_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if let Some(pos) = line.find("-}") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() && !rest.trim().starts_with("--") {
                *count += 1;
            }
        }
        return;
    }

    if let Some(block_start) = line.find("{-") {
        let before = &line[..block_start];
        let has_code_before = !before.trim().is_empty() && !before.trim().starts_with("--");

        if let Some(end_offset) = line[block_start + 2..].find("-}") {
            let after = &line[block_start + 2 + end_offset + 2..];
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

    if line.starts_with("--") {
        return;
    }

    *count += 1;
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
