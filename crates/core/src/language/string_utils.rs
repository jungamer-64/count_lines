// crates/infra/src/measurement/strategies/sloc_counter/string_utils.rs
//! 文字列リテラル検出ユーティリティ
//!
//! 各種言語の文字列リテラル（通常文字列、Raw文字列、バイト文字列等）を
//! 正しく認識し、その内部のコメントマーカーを無視するためのユーティリティ。

/// 文字列リテラルスキップオプション
///
/// 言語ごとに有効な文字列構文を指定することで、
/// 他の言語の構文による誤検出を防ぐ
use alloc::borrow::Cow;
use alloc::string::String;

/// Convert a byte slice to a String, replacing invalid UTF-8 with `REPLACEMENT_CHARACTER`.
/// Mimics `String::from_utf8_lossy`.
#[must_use]
pub fn from_utf8_lossy(input: &[u8]) -> Cow<'_, str> {
    match core::str::from_utf8(input) {
        Ok(valid) => Cow::Borrowed(valid),
        Err(_error) => {
            let mut res = String::with_capacity(input.len());
            let mut remaining = input;
            loop {
                match core::str::from_utf8(remaining) {
                    Ok(valid) => {
                        res.push_str(valid);
                        break;
                    }
                    Err(e) => {
                        let (valid, after_valid) = remaining.split_at(e.valid_up_to());
                        // SAFETY: valid_up_to returns index of first invalid byte, so valid is valid utf8
                        res.push_str(unsafe { core::str::from_utf8_unchecked(valid) });
                        res.push(core::char::REPLACEMENT_CHARACTER);

                        if let Some(chunk_len) = e.error_len() {
                            remaining = &after_valid[chunk_len..];
                        } else {
                            // End of input is invalid
                            break;
                        }
                    }
                }
            }
            Cow::Owned(res)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StringSkipOptions {
    flags: u16,
}

impl StringSkipOptions {
    const RUST_RAW_STRING: u16 = 1 << 0;
    const RUST_BYTE_STRING: u16 = 1 << 1;
    const RUST_LIFETIME: u16 = 1 << 2;
    const CPP_RAW_STRING: u16 = 1 << 3;
    const CSHARP_VERBATIM: u16 = 1 << 4;
    const TEXT_BLOCK: u16 = 1 << 5;
    const BACKTICK_STRING: u16 = 1 << 6;
    const DOUBLE_QUOTE: u16 = 1 << 7;
    const SINGLE_QUOTE: u16 = 1 << 8;
    const REGEX_LITERAL: u16 = 1 << 9;

    #[must_use]
    pub const fn rust_raw_string(self) -> bool {
        self.flags & Self::RUST_RAW_STRING != 0
    }
    #[must_use]
    pub const fn rust_byte_string(self) -> bool {
        self.flags & Self::RUST_BYTE_STRING != 0
    }
    #[must_use]
    pub const fn rust_lifetime(self) -> bool {
        self.flags & Self::RUST_LIFETIME != 0
    }
    #[must_use]
    pub const fn cpp_raw_string(self) -> bool {
        self.flags & Self::CPP_RAW_STRING != 0
    }
    #[must_use]
    pub const fn csharp_verbatim(self) -> bool {
        self.flags & Self::CSHARP_VERBATIM != 0
    }
    #[must_use]
    pub const fn text_block(self) -> bool {
        self.flags & Self::TEXT_BLOCK != 0
    }
    #[must_use]
    pub const fn backtick_string(self) -> bool {
        self.flags & Self::BACKTICK_STRING != 0
    }
    #[must_use]
    pub const fn double_quote(self) -> bool {
        self.flags & Self::DOUBLE_QUOTE != 0
    }
    #[must_use]
    pub const fn single_quote(self) -> bool {
        self.flags & Self::SINGLE_QUOTE != 0
    }
    #[must_use]
    pub const fn regex_literal(self) -> bool {
        self.flags & Self::REGEX_LITERAL != 0
    }

    #[must_use]
    pub const fn with_flag(mut self, flag: u16) -> Self {
        self.flags |= flag;
        self
    }

    /// Rust 用オプション
    #[must_use]
    pub fn rust() -> Self {
        Self::default()
            .with_flag(Self::RUST_RAW_STRING)
            .with_flag(Self::RUST_BYTE_STRING)
            .with_flag(Self::RUST_LIFETIME)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// C/C++ 用オプション (Raw String対応)
    #[must_use]
    pub fn cpp() -> Self {
        Self::default()
            .with_flag(Self::CPP_RAW_STRING)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// C 用オプション (Raw String なし)
    #[must_use]
    pub fn c() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// C# 用オプション (Verbatim String @"..." 対応)
    #[must_use]
    pub fn csharp() -> Self {
        Self::default()
            .with_flag(Self::CSHARP_VERBATIM)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Java/Kotlin/Scala 用オプション (Text Block """...""" 対応)
    #[must_use]
    pub fn java_kotlin() -> Self {
        Self::default()
            .with_flag(Self::TEXT_BLOCK)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Go 用オプション (バッククォート `...` 対応、正規表現リテラルなし)
    #[must_use]
    pub fn go() -> Self {
        Self::default()
            .with_flag(Self::BACKTICK_STRING)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// JavaScript/TypeScript 用オプション (バッククォート `...` と正規表現 /.../ 対応)
    #[must_use]
    pub fn javascript() -> Self {
        Self::default()
            .with_flag(Self::BACKTICK_STRING)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
            .with_flag(Self::REGEX_LITERAL)
    }

    /// Ruby 用オプション (正規表現 /.../ 対応)
    #[must_use]
    pub fn ruby() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
            .with_flag(Self::REGEX_LITERAL)
    }

    /// Perl 用オプション (正規表現 /.../ 対応)
    #[must_use]
    pub fn perl() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
            .with_flag(Self::REGEX_LITERAL)
    }

    /// Swift 用オプション (拡張デリミタ #"..."# 対応)
    ///
    /// Swift固有の文字列:
    /// - 通常: `"..."`
    /// - 多重引用符: `"""..."""`
    /// - 拡張デリミタ: `#"..."#`, `##"..."##`
    #[must_use]
    pub fn swift() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Verilog/SystemVerilog 用オプション
    ///
    /// Verilog は C風の文字列のみ (Raw String なし)
    #[must_use]
    pub fn verilog() -> Self {
        Self::default().with_flag(Self::DOUBLE_QUOTE)
    }

    /// Dart 用オプション
    ///
    /// Dart はバッククォートなし、三重クォートあり
    #[must_use]
    pub fn dart() -> Self {
        Self::default()
            .with_flag(Self::TEXT_BLOCK)
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// Objective-C 用オプション
    ///
    /// @"..." 形式の `NSString` リテラルがあるが、
    /// C# Verbatim String とは異なりエスケープ可能
    #[must_use]
    pub fn objc() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// 基本的な C スタイル (多くの言語で共通)
    #[must_use]
    pub fn basic() -> Self {
        Self::default()
            .with_flag(Self::DOUBLE_QUOTE)
            .with_flag(Self::SINGLE_QUOTE)
    }

    /// 拡張子から適切なオプションを取得
    #[allow(clippy::wildcard_in_or_patterns)]
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            // Rust
            "rs" => Self::rust(),

            // C/C++
            "cpp" | "cc" | "cxx" | "c++" | "hpp" | "hh" | "hxx" | "h++" => Self::cpp(),
            "c" | "h" => Self::c(),

            // C#
            "cs" => Self::csharp(),

            // Java/Kotlin/Scala/Groovy
            "java" | "kt" | "kts" | "scala" | "sc" | "groovy" | "gradle" => Self::java_kotlin(),

            // Go (バッククォート対応、正規表現リテラルなし)
            "go" => Self::go(),

            // JavaScript/TypeScript (バッククォート + 正規表現リテラル対応)
            "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "mts" | "cts" => Self::javascript(),

            // Swift
            "swift" => Self::swift(),

            // Dart
            "dart" => Self::dart(),

            // Objective-C
            "m" | "mm" => Self::objc(),

            // Verilog/SystemVerilog
            "v" | "sv" | "svh" => Self::verilog(),

            // Ruby (正規表現リテラル対応)
            "rb" | "rake" | "gemspec" | "podspec" | "jbuilder" | "erb" => Self::ruby(),

            // Perl (正規表現リテラル対応)
            "pl" | "pm" | "t" | "psgi" => Self::perl(),

            // C-Style Basic (D, Zig, Proto, CSS, etc.)
            "d" | "proto" | "thrift" | "sol" | "ld" | "lds" | "zig" | "css" | "scss" | "sass"
            | "less" | _ => Self::basic(),
        }
    }
}

/// 識別子に使える文字かどうかを判定
#[inline]
#[must_use]
pub const fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn try_skip_prefixed_string(line: &[u8], i: usize, options: StringSkipOptions) -> Option<usize> {
    let bytes = &line[i..];

    if options.csharp_verbatim() && bytes.len() >= 2 && bytes[0] == b'@' && bytes[1] == b'"' {
        return try_skip_csharp_verbatim_string(bytes);
    }

    if options.cpp_raw_string() && bytes.len() >= 2 && bytes[0] == b'R' && bytes[1] == b'"' {
        return try_skip_cpp_raw_string(bytes);
    }

    if options.rust_raw_string() && bytes[0] == b'r' && (i == 0 || !is_ident_char(line[i - 1])) {
        return try_skip_raw_string(bytes);
    }

    if options.rust_byte_string() && bytes[0] == b'b' && (i == 0 || !is_ident_char(line[i - 1])) {
        return try_skip_byte_string(bytes);
    }

    None
}

fn try_skip_quoted_string(line: &[u8], i: usize, options: StringSkipOptions) -> Option<usize> {
    let bytes = &line[i..];
    let b = bytes[0];

    if options.text_block() && b == b'"' && bytes.len() >= 3 && bytes[1] == b'"' && bytes[2] == b'"'
    {
        return try_skip_text_block(bytes);
    }

    if options.double_quote() && b == b'"' {
        let mut j = 1;
        while j < bytes.len() {
            if bytes[j] == b'\\' && j + 1 < bytes.len() {
                j += 2;
                continue;
            }
            if bytes[j] == b'"' {
                return Some(j + 1);
            }
            j += 1;
        }
        return Some(bytes.len());
    }

    if options.single_quote() && b == b'\'' {
        if options.rust_lifetime() {
            if let Some(skip) = try_skip_char_literal(bytes) {
                return Some(skip);
            }
            // Lifetime annotation, just skip '
            return Some(1);
        }

        let mut j = 1;
        while j < bytes.len() {
            if bytes[j] == b'\\' && j + 1 < bytes.len() {
                j += 2;
                continue;
            }
            if bytes[j] == b'\'' {
                return Some(j + 1);
            }
            j += 1;
        }
        return Some(bytes.len());
    }

    if options.backtick_string() && b == b'`' {
        let mut j = 1;
        while j < bytes.len() {
            if bytes[j] == b'\\' && j + 1 < bytes.len() {
                j += 2;
                continue;
            }
            if bytes[j] == b'`' {
                return Some(j + 1);
            }
            j += 1;
        }
        return Some(bytes.len());
    }

    None
}

fn try_skip_regex(line: &[u8], i: usize, options: StringSkipOptions) -> Option<usize> {
    if options.regex_literal() && line[i] == b'/' {
        let is_line_comment = i + 1 < line.len() && line[i + 1] == b'/';
        let is_block_comment = i + 1 < line.len() && line[i + 1] == b'*';

        if !is_line_comment && !is_block_comment {
            return try_skip_regex_literal(line, i);
        }
    }
    None
}

/// 文字列リテラル外でパターンを検索 (言語非依存・基本版)
///
/// 基本的な文字列リテラル ("...", '...') のみをスキップ。
/// 言語固有の文字列構文は考慮しない。
///
/// より正確な検索が必要な場合は `find_outside_string_with_options` を使用。
#[must_use]
pub fn find_outside_string(line: &str, pattern: &str) -> Option<usize> {
    find_outside_string_with_options(line, pattern, StringSkipOptions::rust())
}

/// 文字列リテラル外でパターンを検索 (オプション指定版)
///
/// 指定されたオプションに基づいて、言語固有の文字列構文をスキップする。
/// これにより、異なる言語間での誤検出を防ぐ。
#[must_use]
pub fn find_outside_string_with_options(
    line: &str,
    pattern: &str,
    options: StringSkipOptions,
) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();

    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        if let Some(skip) = try_skip_prefixed_string(line_bytes, i, options) {
            i += skip;
            continue;
        }

        if let Some(skip) = try_skip_quoted_string(line_bytes, i, options) {
            i += skip;
            continue;
        }

        if let Some(skip) = try_skip_regex(line_bytes, i, options) {
            i += skip;
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

/// C++ Raw String Literal をスキップ
/// 形式: R"delimiter(...)delimiter" (delimiterは0-16文字の英数字)
#[must_use]
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
            if remaining.len() > delimiter.len()
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
#[must_use]
pub fn find_hash_outside_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Python の文字列プレフィックスをチェック
        // f, F, u, U, r, R, b, B の組み合わせ (最大2文字)
        if is_python_string_prefix(bytes[i])
            && let Some(skip) = try_skip_python_string(&bytes[i..])
        {
            i += skip;
            continue;
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
                    if i + 2 < bytes.len()
                        && bytes[i] == quote
                        && bytes[i + 1] == quote
                        && bytes[i + 2] == quote
                    {
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
const fn is_python_string_prefix(b: u8) -> bool {
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
            if i + 2 < bytes.len()
                && bytes[i] == quote
                && bytes[i + 1] == quote
                && bytes[i + 2] == quote
            {
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
#[must_use]
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
#[must_use]
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
            if remaining.len() >= hash_count && remaining[..hash_count].iter().all(|&b| b == b'#') {
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
#[must_use]
pub fn find_outside_string_swift(line: &str, pattern: &str) -> Option<usize> {
    let pattern_bytes = pattern.as_bytes();
    let line_bytes = line.as_bytes();

    if pattern_bytes.is_empty() || line_bytes.len() < pattern_bytes.len() {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - pattern_bytes.len() {
        // Swift 拡張デリミタ文字列: #"..."#, ##"..."##, #"""..."""# など
        if line_bytes[i] == b'#'
            && let Some(skip) = try_skip_swift_string(&line_bytes[i..])
        {
            i += skip;
            continue;
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
#[must_use]
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
#[must_use]
pub fn try_skip_text_block(bytes: &[u8]) -> Option<usize> {
    // 最低でも6文字必要: """ + """
    if bytes.len() < 6 || bytes[0] != b'"' || bytes[1] != b'"' || bytes[2] != b'"' {
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
        if bytes[i] == b'"' && i + 2 < bytes.len() && bytes[i + 1] == b'"' && bytes[i + 2] == b'"' {
            return Some(i + 3);
        }
        i += 1;
    }

    // 行末まで閉じられていない (複数行文字列の途中)
    Some(bytes.len())
}

// ============================================================================
// 正規表現リテラルのスキップ
// ============================================================================

/// 正規表現リテラルの開始かどうかを判定
///
/// `/` が正規表現リテラルの開始である可能性が高いかを、
/// 直前のトークンを分析して判定します。
///
/// # 引数
/// * `bytes` - 行全体のバイト列
/// * `pos` - `/` の位置
///
/// # 戻り値
/// * `true` - 正規表現リテラルの可能性が高い
/// * `false` - 除算演算子の可能性が高い
fn is_likely_regex_start(bytes: &[u8], pos: usize) -> bool {
    if pos == 0 {
        // 行頭の `/` は正規表現の可能性が高い
        return true;
    }

    // 直前の非空白文字を探す
    let mut prev_pos = pos;
    while prev_pos > 0 {
        prev_pos -= 1;
        let c = bytes[prev_pos];
        if c != b' ' && c != b'\t' && c != b'\r' && c != b'\n' {
            break;
        }
    }

    // 空白しかなければ正規表現の可能性が高い
    if prev_pos == 0 && (bytes[0] == b' ' || bytes[0] == b'\t') {
        return true;
    }

    let prev_char = bytes[prev_pos];

    // 正規表現が続く可能性が高い文字
    // これらの後の `/` は正規表現の開始である可能性が高い
    match prev_char {
        // 演算子・区切り文字・改行
        b'=' | b'!' | b'(' | b'[' | b'{' | b',' | b';' | b':' | b'?' | b'&' | b'|' | b'^'
        | b'~' | b'<' | b'>' | b'+' | b'-' | b'*' | b'%' | b'\n' | b'\r' => true,

        // 識別子の終わりかキーワードかをチェック
        b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
            // キーワードの後は正規表現の可能性が高い
            is_regex_preceding_keyword(bytes, prev_pos)
        }

        // その他（数字、閉じ括弧など）は除算と判定
        _ => false,
    }
}

/// 正規表現が続く可能性のあるキーワードかどうかを判定
///
/// # 引数
/// * `bytes` - 行全体のバイト列
/// * `end_pos` - キーワードの最後の文字の位置
fn is_regex_preceding_keyword(bytes: &[u8], end_pos: usize) -> bool {
    // キーワードの開始位置を探す
    let mut start_pos = end_pos;
    while start_pos > 0 {
        let prev = bytes[start_pos - 1];
        if prev.is_ascii_alphanumeric() || prev == b'_' {
            start_pos -= 1;
        } else {
            break;
        }
    }

    let keyword = &bytes[start_pos..=end_pos];

    // 正規表現が続く可能性のあるキーワード
    // JavaScript/TypeScript/Ruby/Perl で共通的に使用されるもの
    matches!(
        keyword,
        b"return"
            | b"if"
            | b"else"
            | b"while"
            | b"for"
            | b"do"
            | b"switch"
            | b"case"
            | b"throw"
            | b"new"
            | b"delete"
            | b"typeof"
            | b"void"
            | b"in"
            | b"of"
            | b"instanceof"
            | b"yield"
            | b"await"
            | b"when"
            | b"unless"
            | b"until"
            | b"and"
            | b"or"
            | b"not"
            | b"eq"
            | b"ne"
            | b"lt"
            | b"gt"
            | b"le"
            | b"ge"
            | b"cmp"
            | b"split"
            | b"match"
            | b"grep"
            | b"map"
            | b"sub"
            | b"gsub"
            | b"scan"
            | b"replace"
            | b"test"
            | b"exec"
    )
}

/// 正規表現リテラルをスキップする
///
/// JavaScript/TypeScript/Ruby/Perl の正規表現リテラル `/pattern/flags` を
/// スキップします。
///
/// # 引数
/// * `bytes` - 行全体のバイト列
/// * `start_pos` - `/` の開始位置
///
/// # 戻り値
/// * `Some(n)` - スキップするバイト数
/// * `None` - 正規表現リテラルではない
#[must_use]
pub fn try_skip_regex_literal(bytes: &[u8], start_pos: usize) -> Option<usize> {
    // 正規表現の開始として妥当かチェック
    if !is_likely_regex_start(bytes, start_pos) {
        return None;
    }

    let mut i = start_pos + 1; // 開始の `/` をスキップ

    // 空の正規表現 `//` は除算と区別がつかないのでスキップしない
    if i >= bytes.len() || bytes[i] == b'/' {
        return None;
    }

    // 閉じる `/` を探す
    let mut in_char_class = false; // `[...]` 内かどうか

    while i < bytes.len() {
        let c = bytes[i];

        match c {
            // バックスラッシュエスケープ
            b'\\' => {
                i += 1;
                if i < bytes.len() {
                    i += 1;
                }
            }

            // 文字クラスの開始
            b'[' if !in_char_class => {
                in_char_class = true;
                i += 1;
            }

            // 文字クラスの終了
            b']' if in_char_class => {
                in_char_class = false;
                i += 1;
            }

            // 閉じる `/`（文字クラス外のみ）
            b'/' if !in_char_class => {
                i += 1;
                // フラグをスキップ (g, i, m, s, u, y, d など)
                while i < bytes.len() && is_regex_flag_char(bytes[i]) {
                    i += 1;
                }
                return Some(i - start_pos);
            }

            // 行末に達した場合（正規表現は単一行）
            b'\n' | b'\r' => {
                return None;
            }

            _ => {
                i += 1;
            }
        }
    }

    // 閉じる `/` が見つからなかった
    None
}

/// 正規表現フラグ文字かどうかを判定
#[inline]
const fn is_regex_flag_char(b: u8) -> bool {
    // JavaScript: g, i, m, s, u, y, d, v
    // Ruby: i, m, x, o, e, s, u, n
    // Perl: g, i, m, s, x, o, p, a, d, l, u, n, c, e, r
    b.is_ascii_lowercase()
}

/// SQL 文字列リテラル外でパターンを検索
///
/// SQL の文字列リテラル:
/// - シングルクォート: '...' ('' でエスケープ)
/// - ダブルクォート識別子: "..." (一部のDBでは文字列として使用)
#[must_use]
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

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 正規表現リテラルのテスト
    // =========================================================================

    mod regex_literal_tests {
        use super::*;

        #[test]
        fn test_simple_regex_literal() {
            // 単純な正規表現リテラル
            let bytes = b"/abc/";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_with_flags() {
            // フラグ付き正規表現
            let bytes = b"/abc/gi";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(7));
        }

        #[test]
        fn test_regex_with_escaped_slash() {
            // エスケープされたスラッシュ
            let bytes = b"/a\\/b/";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(6));
        }

        #[test]
        fn test_regex_with_character_class() {
            // 文字クラス内のスラッシュ
            let bytes = b"/[/]/";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_containing_comment_like_pattern() {
            // 正規表現内の // パターン
            let bytes = b"/https:\\/\\//";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(12));
        }

        #[test]
        fn test_division_after_number() {
            // 数値の後の除算
            let bytes = b"10/2";
            let result = try_skip_regex_literal(bytes, 2);
            assert_eq!(result, None);
        }

        #[test]
        fn test_division_after_identifier() {
            // 識別子の後の除算
            let bytes = b"x/2";
            let result = try_skip_regex_literal(bytes, 1);
            assert_eq!(result, None);
        }

        #[test]
        fn test_division_after_closing_paren() {
            // 閉じ括弧の後の除算
            let bytes = b"(x+y)/2";
            let result = try_skip_regex_literal(bytes, 5);
            assert_eq!(result, None);
        }

        #[test]
        fn test_regex_after_equals() {
            // = の後の正規表現
            let bytes = b"x = /abc/";
            let result = try_skip_regex_literal(bytes, 4);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_after_return() {
            // return の後の正規表現
            let bytes = b"return /abc/";
            let result = try_skip_regex_literal(bytes, 7);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_regex_after_open_paren() {
            // ( の後の正規表現
            let bytes = b"if (/abc/.test(s))";
            let result = try_skip_regex_literal(bytes, 4);
            assert_eq!(result, Some(5));
        }

        #[test]
        fn test_empty_regex_treated_as_division() {
            // 空の正規表現は除算として扱う
            let bytes = b"//";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, None);
        }

        #[test]
        fn test_regex_at_line_start() {
            // 行頭の正規表現
            let bytes = b"/abc/g.test(x)";
            let result = try_skip_regex_literal(bytes, 0);
            assert_eq!(result, Some(6));
        }
    }

    // =========================================================================
    // find_outside_string_with_options のテスト (JavaScript/正規表現)
    // =========================================================================

    mod find_outside_string_js_tests {
        use super::*;

        #[test]
        fn test_js_regex_not_mistaken_for_comment() {
            // 正規表現内の // が行コメントと誤認されないこと
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var re = /https:\\/\\//;", "//", options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_js_comment_after_regex() {
            // 正規表現の後の行コメント
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var re = /abc/g; // comment", "//", options);
            assert_eq!(result, Some(17));
        }

        #[test]
        fn test_js_block_comment_in_regex() {
            // 正規表現内の /* */ がブロックコメントと誤認されないこと
            let options = StringSkipOptions::javascript();
            let result = find_outside_string_with_options("var re = /a*b/g;", "/*", options);
            assert_eq!(result, None);
        }

        #[test]
        fn test_js_division_not_regex() {
            // 除算演算子の後のコメント
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var x = a/b; // division", "//", options);
            assert_eq!(result, Some(13));
        }

        #[test]
        fn test_js_template_string_with_regex() {
            // テンプレート文字列内の正規表現パターン
            let options = StringSkipOptions::javascript();
            let result =
                find_outside_string_with_options("var s = `pattern: /abc/`;", "//", options);
            assert_eq!(result, None);
        }
    }
}
