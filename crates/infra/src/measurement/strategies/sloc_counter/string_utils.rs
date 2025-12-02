// crates/infra/src/measurement/strategies/sloc_counter/string_utils.rs
//! 文字列リテラル検出ユーティリティ
//!
//! 各種言語の文字列リテラル（通常文字列、Raw文字列、バイト文字列等）を
//! 正しく認識し、その内部のコメントマーカーを無視するためのユーティリティ。

/// 文字列リテラルスキップオプション
///
/// 言語ごとに有効な文字列構文を指定することで、
/// 他の言語の構文による誤検出を防ぐ
#[derive(Debug, Clone, Copy, Default)]
pub struct StringSkipOptions {
    /// Rust raw string: r"...", r#"..."#
    pub rust_raw_string: bool,
    /// Rust byte string: b"...", br"..."
    pub rust_byte_string: bool,
    /// Rust ライフタイム注釈を考慮 ('a, 'static)
    pub rust_lifetime: bool,
    /// C++ Raw String: R"delimiter(...)delimiter"
    pub cpp_raw_string: bool,
    /// C# Verbatim String: @"..."
    pub csharp_verbatim: bool,
    /// Java/Kotlin Text Block: """..."""
    pub text_block: bool,
    /// バッククォート文字列: `...` (Go, JS/TS)
    pub backtick_string: bool,
    /// 通常のダブルクォート文字列: "..."
    pub double_quote: bool,
    /// 通常のシングルクォート: '...'
    pub single_quote: bool,
}

impl StringSkipOptions {
    /// Rust 用オプション
    pub fn rust() -> Self {
        Self {
            rust_raw_string: true,
            rust_byte_string: true,
            rust_lifetime: true,
            double_quote: true,
            single_quote: true, // 文字リテラル
            ..Default::default()
        }
    }

    /// C/C++ 用オプション
    pub fn cpp() -> Self {
        Self {
            cpp_raw_string: true,
            double_quote: true,
            single_quote: true,
            ..Default::default()
        }
    }

    /// C 用オプション (Raw String なし)
    pub fn c() -> Self {
        Self {
            double_quote: true,
            single_quote: true,
            ..Default::default()
        }
    }

    /// C#/Java/Kotlin 用オプション
    pub fn csharp_java() -> Self {
        Self {
            csharp_verbatim: true,
            text_block: true,
            double_quote: true,
            single_quote: true,
            ..Default::default()
        }
    }

    /// Go/JavaScript/TypeScript 用オプション
    pub fn go_js() -> Self {
        Self {
            backtick_string: true,
            double_quote: true,
            single_quote: true,
            ..Default::default()
        }
    }

    /// 基本的な C スタイル (多くの言語で共通)
    pub fn basic() -> Self {
        Self {
            double_quote: true,
            single_quote: true,
            ..Default::default()
        }
    }
}

/// 識別子に使える文字かどうかを判定
#[inline]
pub fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// 文字列リテラル外でパターンを検索 (言語非依存・基本版)
/// 
/// 基本的な文字列リテラル ("...", '...') のみをスキップ。
/// 言語固有の文字列構文は考慮しない。
/// 
/// より正確な検索が必要な場合は `find_outside_string_with_options` を使用。
pub fn find_outside_string(line: &str, pattern: &str) -> Option<usize> {
    find_outside_string_with_options(line, pattern, &StringSkipOptions::rust())
}

/// 文字列リテラル外でパターンを検索 (オプション指定版)
/// 
/// 指定されたオプションに基づいて、言語固有の文字列構文をスキップする。
/// これにより、異なる言語間での誤検出を防ぐ。
pub fn find_outside_string_with_options(
    line: &str,
    pattern: &str,
    options: &StringSkipOptions,
) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();
    
    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // C# Verbatim String: @"..."
        if options.csharp_verbatim 
            && line_bytes[i] == b'@' 
            && i + 1 < line_bytes.len() 
            && line_bytes[i + 1] == b'"' 
        {
            if let Some(skip) = try_skip_csharp_verbatim_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // C++ Raw String: R"delimiter(...)delimiter"
        if options.cpp_raw_string 
            && line_bytes[i] == b'R' 
            && i + 1 < line_bytes.len() 
            && line_bytes[i + 1] == b'"' 
        {
            if let Some(skip) = try_skip_cpp_raw_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // Rust raw文字列リテラル: r"...", r#"..."#, r##"..."## など
        if options.rust_raw_string
            && line_bytes[i] == b'r'
            && (i == 0 || !is_ident_char(line_bytes[i - 1]))
            && let Some(skip) = try_skip_raw_string(&line_bytes[i..])
        {
            i += skip;
            continue;
        }
        
        // Rust byte文字列: b"...", br"...", br#"..."# など
        if options.rust_byte_string
            && line_bytes[i] == b'b'
            && (i == 0 || !is_ident_char(line_bytes[i - 1]))
        {
            if let Some(skip) = try_skip_byte_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // Java/Kotlin Text Block: """...""" (三重クォート)
        // 通常のダブルクォート処理より先にチェック
        if options.text_block 
            && line_bytes[i] == b'"' 
            && i + 2 < line_bytes.len() 
            && line_bytes[i + 1] == b'"' 
            && line_bytes[i + 2] == b'"' 
        {
            if let Some(skip) = try_skip_text_block(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // ダブルクォート文字列リテラル: "..."
        if options.double_quote && line_bytes[i] == b'"' {
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
        
        // シングルクォート処理
        if options.single_quote && line_bytes[i] == b'\'' {
            if options.rust_lifetime {
                // Rust: 文字リテラル vs ライフタイム注釈の区別
                if let Some(skip) = try_skip_char_literal(&line_bytes[i..]) {
                    i += skip;
                    continue;
                }
                // ライフタイム注釈の場合は単に次へ進む
                i += 1;
                continue;
            } else {
                // 他の言語: 通常のシングルクォート文字列/文字リテラル
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
        }
        
        // バッククォート文字列 (Go Raw String, JS/TS Template Literal)
        if options.backtick_string && line_bytes[i] == b'`' {
            i += 1;
            while i < line_bytes.len() {
                // JS/TSのテンプレートリテラルではエスケープが可能
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2;
                    continue;
                }
                if line_bytes[i] == b'`' {
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
/// C# Verbatim String と Java/Kotlin Text Block にも対応
pub fn find_outside_string_cpp(line: &str, pattern: &str) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();
    
    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // C# Verbatim String: @"..."
        if line_bytes[i] == b'@' && i + 1 < line_bytes.len() && line_bytes[i + 1] == b'"' {
            if let Some(skip) = try_skip_csharp_verbatim_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // C++ Raw String: R"delimiter(...)delimiter"
        if line_bytes[i] == b'R' && i + 1 < line_bytes.len() && line_bytes[i + 1] == b'"' {
            if let Some(skip) = try_skip_cpp_raw_string(&line_bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // Java/Kotlin Text Block: """...""" (三重クォート)
        // 通常のダブルクォート処理より先にチェック
        if line_bytes[i] == b'"' 
            && i + 2 < line_bytes.len() 
            && line_bytes[i + 1] == b'"' 
            && line_bytes[i + 2] == b'"' 
        {
            if let Some(skip) = try_skip_text_block(&line_bytes[i..]) {
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
        
        // バッククォート文字列 (Go Raw String, JS/TS Template Literal)
        if line_bytes[i] == b'`' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\\' && i + 1 < line_bytes.len() {
                    i += 2;
                    continue;
                }
                if line_bytes[i] == b'`' {
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
///
/// Python の文字列プレフィックスを考慮:
/// - f-string: `f"..."`, `F"..."`
/// - Unicode: `u"..."`, `U"..."`
/// - Raw: `r"..."`, `R"..."`
/// - Bytes: `b"..."`, `B"..."`
/// - 複合: `fr"..."`, `rf"..."`, `br"..."`, `rb"..."` など
pub fn find_hash_outside_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // Python の文字列プレフィックスをチェック
        // f, F, u, U, r, R, b, B の組み合わせ (最大2文字)
        if is_python_string_prefix(bytes[i]) {
            if let Some(skip) = try_skip_python_string(&bytes[i..]) {
                i += skip;
                continue;
            }
        }
        
        // 文字列: "..." または '...'
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            
            // 三重引用符かチェック
            if i + 1 < bytes.len() && bytes[i] == quote && bytes[i + 1] == quote {
                // 三重引用符: """...""" または '''...'''
                i += 2; // 開始の3文字目
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() {
                        i += 2;
                        continue;
                    }
                    if i + 2 < bytes.len() && bytes[i] == quote && bytes[i + 1] == quote && bytes[i + 2] == quote {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            
            // 通常の文字列
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

/// Python の文字列プレフィックス文字かどうかを判定
#[inline]
fn is_python_string_prefix(b: u8) -> bool {
    matches!(b, b'f' | b'F' | b'u' | b'U' | b'r' | b'R' | b'b' | b'B')
}

/// Python の文字列リテラルをスキップ (プレフィックス対応)
///
/// 対応するプレフィックス:
/// - 単独: f, F, u, U, r, R, b, B
/// - 複合: fr, rf, Fr, rF, FR, RF, fR, Rf, br, rb, Br, rB, BR, RB, bR, Rb
fn try_skip_python_string(bytes: &[u8]) -> Option<usize> {
    if bytes.is_empty() || !is_python_string_prefix(bytes[0]) {
        return None;
    }
    
    let mut prefix_len = 1;
    
    // 2文字目もプレフィックスかチェック (fr, rf, br, rb など)
    if bytes.len() > 1 && is_python_string_prefix(bytes[1]) {
        // 有効な組み合わせかチェック
        let first = bytes[0].to_ascii_lowercase();
        let second = bytes[1].to_ascii_lowercase();
        
        // 有効な2文字プレフィックス: fr, rf, br, rb
        if (first == b'f' && second == b'r')
            || (first == b'r' && second == b'f')
            || (first == b'b' && second == b'r')
            || (first == b'r' && second == b'b')
        {
            prefix_len = 2;
        }
    }
    
    // プレフィックスの後にクォートがあるか確認
    if bytes.len() <= prefix_len {
        return None;
    }
    
    let quote_start = prefix_len;
    let quote = bytes[quote_start];
    if quote != b'"' && quote != b'\'' {
        return None;
    }
    
    let mut i = quote_start + 1;
    
    // 三重引用符かチェック
    let is_triple = i + 1 < bytes.len() && bytes[i] == quote && bytes[i + 1] == quote;
    if is_triple {
        i += 2; // 開始の3文字目
        while i < bytes.len() {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if i + 2 < bytes.len() && bytes[i] == quote && bytes[i + 1] == quote && bytes[i + 2] == quote {
                return Some(i + 3);
            }
            i += 1;
        }
        return Some(bytes.len());
    }
    
    // 通常の文字列
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        if bytes[i] == quote {
            return Some(i + 1);
        }
        i += 1;
    }
    
    Some(bytes.len())
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

/// C# Verbatim String をスキップ
/// 形式: @"..." ( " は "" でエスケープ、\ はエスケープしない)
pub fn try_skip_csharp_verbatim_string(bytes: &[u8]) -> Option<usize> {
    if bytes.len() < 2 || bytes[0] != b'@' || bytes[1] != b'"' {
        return None;
    }
    
    let mut i = 2;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            // ダブルクォート2つ ("") はエスケープされた " 1つとみなす
            if i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                i += 2;
                continue;
            }
            // 単独の " は文字列終了
            return Some(i + 1);
        }
        i += 1;
    }
    
    // 行末まで閉じられていない (C# verbatim string は改行を許可するためこれでOK)
    Some(bytes.len())
}

/// Java/Kotlin Text Block (三重クォート) をスキップ
/// 形式: """..."""
pub fn try_skip_text_block(bytes: &[u8]) -> Option<usize> {
    // 最低でも6文字必要: """ + """
    if bytes.len() < 6 
        || bytes[0] != b'"' 
        || bytes[1] != b'"' 
        || bytes[2] != b'"' 
    {
        return None;
    }
    
    let mut i = 3;
    while i < bytes.len() {
        // エスケープシーケンス
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 2;
            continue;
        }
        
        // 終了の """ を探す
        if bytes[i] == b'"' 
            && i + 2 < bytes.len() 
            && bytes[i + 1] == b'"' 
            && bytes[i + 2] == b'"' 
        {
            return Some(i + 3);
        }
        i += 1;
    }
    
    // 行末まで閉じられていない (複数行文字列の途中)
    Some(bytes.len())
}

/// SQL 文字列リテラル外でパターンを検索
/// 
/// SQL の文字列リテラル:
/// - シングルクォート: '...' ('' でエスケープ)
/// - ダブルクォート識別子: "..." (一部のDBでは文字列として使用)
pub fn find_outside_string_sql(line: &str, pattern: &str) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();
    
    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // シングルクォート文字列: '...' ('' でエスケープ)
        if line_bytes[i] == b'\'' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'\'' {
                    // '' はエスケープされた ' 1つ
                    if i + 1 < line_bytes.len() && line_bytes[i + 1] == b'\'' {
                        i += 2;
                        continue;
                    }
                    // 単独の ' は文字列終了
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // ダブルクォート識別子/文字列: "..." (一部のDBでは "" でエスケープ)
        if line_bytes[i] == b'"' {
            i += 1;
            while i < line_bytes.len() {
                if line_bytes[i] == b'"' {
                    // "" はエスケープされた " 1つ
                    if i + 1 < line_bytes.len() && line_bytes[i + 1] == b'"' {
                        i += 2;
                        continue;
                    }
                    // 単独の " は文字列終了
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
