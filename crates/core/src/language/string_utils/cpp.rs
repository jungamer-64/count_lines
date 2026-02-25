// crates/core/src/language/string_utils/cpp.rs
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
