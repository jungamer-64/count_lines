// crates/core/src/language/string_utils/search.rs
use crate::language::StringSkipOptions;
use crate::language::string_utils::try_skip_prefixed_string;
use crate::language::string_utils::try_skip_quoted_string;
use crate::language::string_utils::try_skip_regex;
use crate::language::string_utils::try_skip_swift_string;

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

/// Result of multi-pattern search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternMatch {
    /// The position where the pattern was found.
    pub position: usize,
    /// Index of the matched pattern in the input array.
    pub pattern_index: usize,
}

/// Find the first occurrence of any pattern outside string literals.
///
/// Returns `None` if no pattern is found.
/// If multiple patterns could match at the same position, returns the one
/// with the smallest index (first in the array).
///
/// # Performance
///
/// - Time: O(n × p) where n = line length, p = number of patterns
/// - Space: O(1)
///
/// This is more efficient than calling `find_outside_string_with_options` multiple times
/// because it scans the line only once.
///
/// # Example
///
/// ```rust,ignore
/// use count_lines_core::language::string_utils::{find_any_outside_string, StringSkipOptions};
///
/// let line = "int x = 1; // comment";
/// let patterns = ["//", "/*"];
/// if let Some(m) = find_any_outside_string(line, &patterns, StringSkipOptions::default()) {
///     match m.pattern_index {
///         0 => println!("Line comment at {}", m.position),
///         1 => println!("Block comment at {}", m.position),
///         _ => {}
///     }
/// }
/// ```
#[must_use]
pub fn find_any_outside_string(
    line: &str,
    patterns: &[&str],
    options: StringSkipOptions,
) -> Option<PatternMatch> {
    let line_bytes = line.as_bytes();

    if patterns.is_empty() || line_bytes.is_empty() {
        return None;
    }

    // Pre-convert patterns to bytes for efficiency
    let pattern_bytes: alloc::vec::Vec<&[u8]> = patterns.iter().map(|p| p.as_bytes()).collect();

    // Find minimum pattern length for early termination
    let min_pattern_len = pattern_bytes.iter().map(|p| p.len()).min().unwrap_or(0);
    if min_pattern_len == 0 || line_bytes.len() < min_pattern_len {
        return None;
    }

    let mut i = 0;
    while i <= line_bytes.len() - min_pattern_len {
        // Skip string literals
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

        // Check all patterns at current position
        for (pattern_index, pattern) in pattern_bytes.iter().enumerate() {
            if i + pattern.len() <= line_bytes.len()
                && &line_bytes[i..i + pattern.len()] == *pattern
            {
                return Some(PatternMatch {
                    position: i,
                    pattern_index,
                });
            }
        }

        i += 1;
    }

    None
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
