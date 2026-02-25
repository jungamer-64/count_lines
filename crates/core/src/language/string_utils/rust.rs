// crates/core/src/language/string_utils/rust.rs
/// Rust raw文字列リテラルをスキップする
///
/// `r"..."`, `r#"..."#`, `r##"..."##` などの形式を処理
/// 成功した場合はスキップするバイト数を返す
#[must_use]
pub fn try_skip_raw_string(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() || bytes[0] != b'r' {
        return None;
    }

    let mut i = 1;

    // '#' の数をカウント
    let mut hash_count = 0;
    while i < bytes.len() && bytes[i] == b'#' {
        hash_count += 1;
        i += 1;
    }

    // '"' で始まる必要がある
    if i >= bytes.len() || bytes[i] != b'"' {
        return None;
    }
    i += 1;

    // 終端の '"' + '#' * hash_count を探す
    while i < bytes.len() {
        if bytes[i] == b'"' {
            // 閉じクォートの後に必要な数の '#' があるか確認
            let remaining = &bytes[i + 1..];
            if hash_count == 0 {
                // r"..." の場合
                return Some(i + 1);
            } else if remaining.len() >= hash_count
                && remaining[..hash_count].iter().all(|&b| b == b'#')
            {
                // r#"..."# や r##"..."## の場合
                return Some(i + 1 + hash_count);
            }
        }
        i += 1;
    }

    // 閉じられていない raw 文字列（行末まで）
    Some(bytes.len())
}

/// Rust byte文字列をスキップする
///
/// `b"..."`, `br"..."`, `br#"..."#` などの形式を処理
/// 成功した場合はスキップするバイト数を返す
#[must_use]
pub fn try_skip_byte_string(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() || bytes[0] != b'b' {
        return None;
    }

    if bytes.len() < 2 {
        return None;
    }

    // b"..." の場合
    if bytes[1] == b'"' {
        let mut i = 2;
        while i < bytes.len() {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                return Some(i + 1);
            }
            i += 1;
        }
        return Some(bytes.len());
    }

    // br"..." または br#"..."# の場合
    if bytes[1] == b'r' {
        // try_skip_raw_string に &bytes[1..] を渡して、+1 して返す
        if let Some(skip) = try_skip_raw_string(&bytes[1..]) {
            return Some(1 + skip);
        }
    }

    None
}

/// 文字リテラルをスキップする（ライフタイム注釈との区別）
///
/// 文字リテラル: `'a'`, `'\n'`, `'\u{1234}'` など（最大8文字程度）
/// ライフタイム: `'a`, `'static` など（閉じクォートがない）
///
/// 閉じクォートが12文字以内に見つからない場合はライフタイムとみなしNoneを返す
#[must_use]
pub fn try_skip_char_literal(bytes: &[u8]) -> Option<usize> {
    const MAX_CHAR_LITERAL_LEN: usize = 12; // '\u{10FFFF}' + 余裕

    if bytes.is_empty() || bytes[0] != b'\'' {
        return None;
    }

    let search_limit = bytes.len().min(MAX_CHAR_LITERAL_LEN);

    let mut i = 1;
    while i < search_limit {
        if bytes[i] == b'\\' && i + 1 < search_limit {
            i += 2; // エスケープシーケンスをスキップ
            continue;
        }
        if bytes[i] == b'\'' {
            // 閉じクォートが見つかった = 文字リテラル
            return Some(i + 1);
        }
        i += 1;
    }

    // 閉じクォートが見つからない = ライフタイム注釈
    None
}
