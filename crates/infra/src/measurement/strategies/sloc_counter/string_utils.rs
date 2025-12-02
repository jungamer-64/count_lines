// crates/infra/src/measurement/strategies/sloc_counter/string_utils.rs
//! 文字列リテラル検出ユーティリティ
//!
//! 各種言語の文字列リテラル（通常文字列、Raw文字列、バイト文字列等）を
//! 正しく認識し、その内部のコメントマーカーを無視するためのユーティリティ。

/// 識別子に使える文字かどうかを判定
#[inline]
pub fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// 文字列リテラル外でパターンを検索 (Rust向け)
/// 
/// 以下の文字列リテラル内のパターンは無視する:
/// - 通常の文字列: `"..."`, `'...'`
/// - Rust raw文字列: `r"..."`, `r#"..."#`, `r##"..."##` など
/// - バイト文字列: `b"..."`, `br"..."`, `br#"..."#` など
pub fn find_outside_string(line: &str, pattern: &str) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();
    
    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // Rust raw文字列リテラル: r"...", r#"..."#, r##"..."## など
        // 直前が識別子文字でないことを確認（bar"..." などを誤検出しない）
        if line_bytes[i] == b'r'
            && (i == 0 || !is_ident_char(line_bytes[i - 1]))
            && let Some(skip) = try_skip_raw_string(&line_bytes[i..])
        {
            i += skip;
            continue;
        }
        
        // Rust byte文字列: b"...", br"...", br#"..."# など
        if line_bytes[i] == b'b'
            && (i == 0 || !is_ident_char(line_bytes[i - 1]))
        {
            if let Some(skip) = try_skip_byte_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // ダブルクォート文字列リテラル: "..."
        if line_bytes[i] == b'"' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2; // エスケープシーケンスをスキップ
                    continue;
                }
                if line_bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // シングルクォート: 文字リテラル ('a', '\n' など) vs ライフタイム ('a)
        // 文字リテラルは短い（最大8文字程度: '\u{10FFFF}'）
        // 閉じクォートが見つからない場合はライフタイムとみなしスキップしない
        if line_bytes[i] == b'\'' {
            if let Some(skip) = try_skip_char_literal(&line_bytes[i..]) {
                i += skip;
                continue;
            }
            // ライフタイム注釈の場合は単に次へ進む
            i += 1;
            continue;
        }
        
        // パターンとマッチするかチェック
        if i + pattern_bytes.len() <= line_bytes.len() 
            && &line_bytes[i..i + pattern_bytes.len()] == pattern_bytes 
        {
            return Some(i);
        }
        
        i += 1;
    }
    
    None
}

/// Rust raw文字列リテラルをスキップする
/// 
/// `r"..."`, `r#"..."#`, `r##"..."##` などの形式を処理
/// 成功した場合はスキップするバイト数を返す
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
pub fn try_skip_char_literal(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() || bytes[0] != b'\'' {
        return None;
    }
    
    const MAX_CHAR_LITERAL_LEN: usize = 12; // '\u{10FFFF}' + 余裕
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

/// C++ Raw String Literal を考慮した文字列外検索
pub fn find_outside_string_cpp(line: &str, pattern: &str) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();
    
    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // C++ Raw String: R"delimiter(...)delimiter"
        if line_bytes[i] == b'R' && i + 1 < line_bytes.len() && line_bytes[i + 1] == b'"' {
            if let Some(skip) = try_skip_cpp_raw_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // 通常の文字列: "..."
        if line_bytes[i] == b'"' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2;
                    continue;
                }
                if line_bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // 文字リテラル: '...'
        if line_bytes[i] == b'\'' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2;
                    continue;
                }
                if line_bytes[i] == b'\'' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // パターンマッチ
        if i + pattern_bytes.len() <= line_bytes.len()
            && &line_bytes[i..i + pattern_bytes.len()] == pattern_bytes
        {
            return Some(i);
        }
        
        i += 1;
    }
    
    None
}

/// C++ Raw String Literal をスキップ
/// 形式: R"delimiter(...)delimiter" (delimiterは0-16文字の英数字)
pub fn try_skip_cpp_raw_string(bytes: &[u8]) -> Option<usize> {
    // R" で始まる必要がある
    if bytes.len() < 3 || bytes[0] != b'R' || bytes[1] != b'"' {
        return None;
    }
    
    let mut i = 2;
    
    // デリミタを取得 (最大16文字)
    let delimiter_start = i;
    while i < bytes.len() && i - delimiter_start < 16 {
        if bytes[i] == b'(' {
            break;
        }
        // デリミタに使える文字: 英数字とアンダースコア（スペースや括弧は除く）
        if !bytes[i].is_ascii_alphanumeric() && bytes[i] != b'_' {
            return None;
        }
        i += 1;
    }
    
    if i >= bytes.len() || bytes[i] != b'(' {
        return None;
    }
    
    let delimiter = &bytes[delimiter_start..i];
    i += 1; // '(' をスキップ
    
    // 終端パターン: )delimiter" を探す
    while i < bytes.len() {
        if bytes[i] == b')' {
            let remaining = &bytes[i + 1..];
            if remaining.len() >= delimiter.len() + 1
                && &remaining[..delimiter.len()] == delimiter
                && remaining[delimiter.len()] == b'"'
            {
                return Some(i + 1 + delimiter.len() + 1);
            }
        }
        i += 1;
    }
    
    // 行末まで閉じられていない
    Some(bytes.len())
}

/// 文字列外の # を検索（Python用）
pub fn find_hash_outside_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // 文字列: "..." または '...'
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        if bytes[i] == b'#' {
            return Some(i);
        }
        
        i += 1;
    }
    
    None
}

/// Docstring開始をチェック（三重クォートで始まるか）
pub fn check_docstring_start(trimmed: &str) -> Option<u8> {
    if trimmed.starts_with("\"\"\"") {
        Some(b'"')
    } else if trimmed.starts_with("'''") {
        Some(b'\'')
    } else {
        None
    }
}

/// Swift の文字列リテラルをスキップ
///
/// Swift には以下の文字列形式がある:
/// - 通常: `"..."`
/// - 多重引用符 (Multiline): `"""..."""`
/// - 拡張デリミタ: `#"..."#`, `##"..."##`
/// - 拡張デリミタ + 多重引用符: `#"""..."""#`
///
/// 成功した場合はスキップするバイト数を返す
pub fn try_skip_swift_string(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() {
        return None;
    }

    // 1. 拡張デリミタ (#の数を数える)
    let mut hash_count = 0;
    let mut i = 0;
    while i < bytes.len() && bytes[i] == b'#' {
        hash_count += 1;
        i += 1;
    }

    // クォートの確認
    if i >= bytes.len() || bytes[i] != b'"' {
        return None; // 文字列ではない
    }
    i += 1; // '"' をスキップ

    // 2. 多重引用符 (""") の確認
    // 注: Swiftでは拡張デリミタと多重引用符は併用可能 ( #"""..."""# )
    let is_multiline = if i + 1 < bytes.len() && bytes[i] == b'"' && bytes[i + 1] == b'"' {
        i += 2; // さらに2つスキップ
        true
    } else {
        false
    };

    // 終端を探す
    while i < bytes.len() {
        if bytes[i] == b'"' {
            // 多重引用符の場合、""" かチェック
            if is_multiline {
                if i + 2 < bytes.len() && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
                    // """ が見つかった。続いてハッシュ数を確認
                    let remaining = &bytes[i + 3..];
                    if remaining.len() >= hash_count
                        && remaining[..hash_count].iter().all(|&b| b == b'#')
                    {
                        return Some(i + 3 + hash_count);
                    }
                    // ハッシュが足りない、または単なる """ の中身 -> スキップして続行
                    i += 1;
                    continue;
                }
            } else {
                // 通常引用符の場合
                let remaining = &bytes[i + 1..];
                if remaining.len() >= hash_count
                    && remaining[..hash_count].iter().all(|&b| b == b'#')
                {
                    return Some(i + 1 + hash_count);
                }
            }
        }

        // エスケープシーケンス (\) のスキップ
        // Swiftではバックスラッシュもハッシュの影響を受ける ( \#( ) など )
        if bytes[i] == b'\\' {
            // エスケープマーカーとして有効かチェック（ハッシュ数が一致するか）
            let remaining = &bytes[i + 1..];
            if remaining.len() >= hash_count
                && remaining[..hash_count].iter().all(|&b| b == b'#')
            {
                // エスケープ有効。ハッシュ分 + 次の文字をスキップ
                i += 1 + hash_count;
                if i < bytes.len() {
                    // 式展開 \#(...) の場合は括弧の中身をスキップする必要があるが、
                    // 簡易的には次の1文字をスキップ
                    i += 1;
                }
                continue;
            }
        }

        i += 1;
    }

    // 閉じられていない文字列（行末まで、または複数行文字列の途中）
    Some(bytes.len())
}

/// Swift の文字列リテラルを考慮した文字列外検索
pub fn find_outside_string_swift(line: &str, pattern: &str) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();

    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // Swift 拡張デリミタ文字列: #"..."#, ##"..."##, #"""..."""# など
        if line_bytes[i] == b'#' {
            if let Some(skip) = try_skip_swift_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }

        // 通常/多重引用符文字列: "...", """..."""
        if line_bytes[i] == b'"' {
            if let Some(skip) = try_skip_swift_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
            // フォールバック: 通常の文字列スキップ
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2;
                    continue;
                }
                if line_bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // 文字リテラル (Swiftでは Character 型で使用は少ないが対応)
        if line_bytes[i] == b'\'' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2;
                    continue;
                }
                if line_bytes[i] == b'\'' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        // パターンとマッチするかチェック
        if i + pattern_bytes.len() <= line_bytes.len()
            && &line_bytes[i..i + pattern_bytes.len()] == pattern_bytes
        {
            return Some(i);
        }

        i += 1;
    }

    None
}
