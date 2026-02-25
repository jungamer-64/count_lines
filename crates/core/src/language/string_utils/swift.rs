// crates/core/src/language/string_utils/swift.rs
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
