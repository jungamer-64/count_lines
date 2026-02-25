// crates/core/src/language/string_utils/skip.rs
use crate::language::StringSkipOptions;
use crate::language::string_utils::try_skip_byte_string;
use crate::language::string_utils::try_skip_char_literal;
use crate::language::string_utils::try_skip_cpp_raw_string;
use crate::language::string_utils::try_skip_raw_string;

/// 識別子に使える文字かどうかを判定
#[inline]
#[must_use]
pub const fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Try to skip a prefixed string literal (e.g. `r"..."`, `b"..."`, `@"..."`, `R"(...)"`).
#[must_use]
pub fn try_skip_prefixed_string(
    line: &[u8],
    i: usize,
    options: StringSkipOptions,
) -> Option<usize> {
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

/// Try to skip a quoted string literal (`"..."`, `'...'`, or backtick).
#[must_use]
pub fn try_skip_quoted_string(line: &[u8], i: usize, options: StringSkipOptions) -> Option<usize> {
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

/// Try to skip a regex literal (`/.../`).
#[must_use]
pub fn try_skip_regex(line: &[u8], i: usize, options: StringSkipOptions) -> Option<usize> {
    if options.regex_literal() && line[i] == b'/' {
        let is_line_comment = i + 1 < line.len() && line[i + 1] == b'/';
        let is_block_comment = i + 1 < line.len() && line[i + 1] == b'*';

        if !is_line_comment && !is_block_comment {
            return try_skip_regex_literal(line, i);
        }
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
