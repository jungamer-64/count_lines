// crates/infra/src/measurement/strategies/sloc_counter/processors/hash_style.rs
//! Hash系言語のコメント処理
//!
//! Python/Ruby/Shell等の `#` コメントを処理します。
//! - Python Docstring（三重クォート `"""` / `'''`）
//! - Ruby 埋め込みドキュメント（`=begin` ～ `=end`）
//! - Perl POD（`=pod` / `=head1` 等 ～ `=cut`）

use super::super::string_utils::{check_docstring_start, find_hash_outside_string};

/// Hash スタイル (#) の処理
/// 
/// 対応するブロックコメント形式:
/// - Python: `"""..."""` / `'''...'''`
/// - Ruby: `=begin` ～ `=end` (行頭必須)
/// - Perl: `=pod` / `=head1` 等 ～ `=cut` (行頭必須)
pub fn process_hash_style(
    line: &str,
    docstring_quote: &mut Option<u8>,
    in_embedded_doc: &mut bool,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // ==================== Ruby/Perl 埋め込みドキュメント処理 ====================
    // 注: Ruby の =begin / Perl の =pod 等は「行頭」から始まる必要がある
    
    // 埋め込みドキュメント内の場合
    if *in_embedded_doc {
        // Ruby: =end / Perl: =cut で終了
        if line.starts_with("=end") || line.starts_with("=cut") {
            *in_embedded_doc = false;
        }
        return; // ドキュメント内はカウントしない
    }

    // 埋め込みドキュメント開始判定（行頭の = で始まる）
    // Ruby: =begin
    // Perl: =pod, =head1, =head2, =over, =item, =back, =encoding, =for, =begin (PODコマンド)
    if line.starts_with("=begin") || is_perl_pod_start(line) {
        *in_embedded_doc = true;
        return;
    }

    // ==================== Python Docstring 処理 ====================
    
    // Docstring内の場合
    if let Some(quote) = *docstring_quote {
        let closing = if quote == b'"' { "\"\"\"" } else { "'''" };
        if line.contains(closing) {
            *docstring_quote = None;
            *in_block_comment = false;
        }
        return;
    }

    // shebang行を除外
    if trimmed.starts_with("#!") && *count == 0 {
        return;
    }
    
    // #で始まる行はコメント
    if trimmed.starts_with('#') {
        return;
    }

    // Python Docstring開始判定（行頭または代入の右辺として現れる三重クォート）
    // 簡易版: trimmed が三重クォートで始まる場合のみDocstring扱い
    if let Some(quote_type) = check_docstring_start(trimmed) {
        let closing = if quote_type == b'"' { "\"\"\"" } else { "'''" };
        // 同じ行で閉じているか確認
        if trimmed.len() > 3 && trimmed[3..].contains(closing) {
            // 1行Docstring -> コメント扱い
            return;
        }
        *docstring_quote = Some(quote_type);
        *in_block_comment = true;
        return;
    }

    // ==================== 通常のコード処理 ====================
    
    // # より前にコードがあるか
    if let Some(hash_pos) = find_hash_outside_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

/// Perl POD (Plain Old Documentation) の開始行かどうかを判定
/// 
/// PODは `=` で始まり、英字が続くコマンドで開始される:
/// - `=pod`, `=head1`, `=head2`, `=head3`, `=head4`
/// - `=over`, `=item`, `=back`
/// - `=encoding`, `=for`, `=begin`, `=end`
fn is_perl_pod_start(line: &str) -> bool {
    if !line.starts_with('=') {
        return false;
    }
    
    let bytes = line.as_bytes();
    if bytes.len() < 2 {
        return false;
    }
    
    // = の次が英字で始まる場合は POD コマンド
    // (=begin は Ruby と共通なので上で処理済み、ここでは =pod, =head 等を検出)
    let second = bytes[1];
    if !second.is_ascii_alphabetic() {
        return false;
    }
    
    // 主要な POD コマンドをチェック
    line.starts_with("=pod")
        || line.starts_with("=head")
        || line.starts_with("=over")
        || line.starts_with("=item")
        || line.starts_with("=back")
        || line.starts_with("=encoding")
        || line.starts_with("=for")
}
