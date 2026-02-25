// crates/core/src/language/string_utils/mod.rs
//! 文字列リテラル検出ユーティリティ
//!
//! 各種言語の文字列リテラル（通常文字列、Raw文字列、バイト文字列等）を
//! 正しく認識し、その内部のコメントマーカーを無視するためのユーティリティ。

/// C/C++関連の文字列スキップ処理
pub mod cpp;
/// 文字列スキップのオプション設定
pub mod options;
/// Python関連の文字列スキップ処理
pub mod python;
/// Rust関連の文字列スキップ処理
pub mod rust;
/// 文字列リテラル外でのパターン検索
pub mod search;
/// 文字列や正規表現のスキップ処理
pub mod skip;
/// Swift関連の文字列スキップ処理
pub mod swift;
#[cfg(test)]
mod tests;

pub use cpp::*;
pub use options::*;
pub use python::*;
pub use rust::*;
pub use search::*;
pub use skip::*;
pub use swift::*;

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
