// crates/core/src/language/string_utils/python.rs
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
