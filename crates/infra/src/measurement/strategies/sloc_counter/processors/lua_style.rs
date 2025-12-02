// crates/infra/src/measurement/strategies/sloc_counter/processors/lua_style.rs
//! Lua言語のコメント処理
//!
//! Lua固有の対応:
//! - 行コメント: `--`
//! - ブロックコメント: `--[[` ～ `]]`
//! - 等号付きブロックコメント: `--[=[` ～ `]=]`, `--[==[` ～ `]==]` 等

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
        if find_lua_block_end(line, *lua_block_level).is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;

    // テストヘルパー: 複数行を処理
    fn process_lines(lines: &[&str]) -> usize {
        let mut count = 0;
        let mut in_block = false;
        let mut level = 0;
        for line in lines {
            process_lua_style(line, &mut in_block, &mut level, &mut count);
        }
        count
    }

    #[test]
    fn test_line_comment() {
        let count = process_lines(&[
            "-- line comment",
            "local b = 2",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment_basic() {
        let count = process_lines(&[
            "--[[",
            "  block comment",
            "]]",
            "local x = 1",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment_level_1() {
        let count = process_lines(&[
            "--[=[",
            "  contains ]] but not end",
            "]=]",
            "local y = 2",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment_level_3() {
        let count = process_lines(&[
            "--[===[",
            "  contains ]] and ]=] but not end",
            "]===]",
            "local z = 3",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_block_comment_single_line() {
        let count = process_lines(&[
            "--[[ single line block ]]",
            "local a = 1",
        ]);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_standard_block_comment() {
        let count = process_lines(&[
            "-- comment",
            "local x = 1",
            "--[[ block",
            "comment ]]",
            "local y = 2",
        ]);
        assert_eq!(count, 2);
    }
}
