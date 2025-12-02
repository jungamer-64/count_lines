// crates/infra/src/measurement/strategies/sloc_counter/processors/hash_style.rs
//! Hash系言語のコメント処理
//!
//! Python/Ruby/Perl/Shell等の `#` コメントを処理します。
//! - Python: Docstring（三重クォート `"""` / `'''`）, f-string等
//! - Ruby: 埋め込みドキュメント（`=begin` ～ `=end`）
//! - Perl: POD（`=pod` / `=head1` 等 ～ `=cut`）
//! - Shell/YAML/Config: 単純な # コメント（複雑な文字列処理不要）

use super::super::string_utils::{check_docstring_start, find_hash_outside_string};

/// Python スタイル (#) の処理
/// 
/// Python固有の対応:
/// - Docstring: `"""..."""` / `'''...'''`
/// - f-string: `f"..."`, `F"..."` 等の文字列プレフィックス
/// - 複合プレフィックス: `fr"..."`, `rf"..."` 等
pub fn process_python_style(
    line: &str,
    docstring_quote: &mut Option<u8>,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

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

    // # より前にコードがあるか (f-string等を考慮)
    if let Some(hash_pos) = find_hash_outside_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

/// Ruby スタイル (#) の処理
/// 
/// Ruby固有の対応:
/// - 埋め込みドキュメント: `=begin` ～ `=end` (行頭必須)
/// - 文字列: `"..."`, `'...'` のみ考慮
pub fn process_ruby_style(
    line: &str,
    in_embedded_doc: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // 埋め込みドキュメント内の場合
    if *in_embedded_doc {
        // Ruby: =end で終了
        if line.starts_with("=end") {
            *in_embedded_doc = false;
        }
        return;
    }

    // 埋め込みドキュメント開始判定（行頭の =begin で始まる）
    if line.starts_with("=begin") {
        *in_embedded_doc = true;
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

    // # より前にコードがあるか (標準的な文字列のみ考慮)
    if let Some(hash_pos) = find_hash_outside_simple_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

/// Perl スタイル (#) の処理
/// 
/// Perl固有の対応:
/// - POD: `=pod`, `=head1` 等 ～ `=cut` (行頭必須)
/// - 文字列: `"..."`, `'...'` のみ考慮
pub fn process_perl_style(
    line: &str,
    in_embedded_doc: &mut bool,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // POD内の場合
    if *in_embedded_doc {
        // Perl: =cut で終了
        if line.starts_with("=cut") {
            *in_embedded_doc = false;
        }
        return;
    }

    // POD開始判定
    if is_perl_pod_start(line) {
        *in_embedded_doc = true;
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

    // # より前にコードがあるか (標準的な文字列のみ考慮)
    if let Some(hash_pos) = find_hash_outside_simple_string(line) {
        let before = &line[..hash_pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
    } else {
        *count += 1;
    }
}

/// 単純な Hash スタイル (#) の処理
/// 
/// 対象: Shell, YAML, TOML, Dockerfile, Makefile, Config系など
/// 
/// 特徴:
/// - 複雑な文字列処理不要
/// - `"..."` と `'...'` のみ考慮（バッククォートや三重クォートなし）
/// - Docstringや埋め込みドキュメントなし
/// - 高速かつ安全な処理
pub fn process_simple_hash_style(
    line: &str,
    count: &mut usize,
) {
    let trimmed = line.trim();

    // shebang行を除外
    if trimmed.starts_with("#!") && *count == 0 {
        return;
    }
    
    // #で始まる行はコメント
    if trimmed.starts_with('#') {
        return;
    }

    // # より前にコードがあるか (単純な文字列のみ考慮)
    if let Some(hash_pos) = find_hash_outside_simple_string(line) {
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

/// 単純な文字列 ("..." / '...') 外で # を検索
/// 
/// Shell/YAML/Config等向けの軽量版。
/// Python の f-string や三重クォートは考慮しない。
fn find_hash_outside_simple_string(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        // ダブルクォート文字列: "..."
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2; // エスケープシーケンスをスキップ
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        
        // シングルクォート文字列: '...'
        if bytes[i] == b'\'' {
            i += 1;
            while i < bytes.len() {
                // シングルクォート内はエスケープなし (シェル的解釈)
                // ただし '' で1つの ' を表す場合があるので、次の文字もチェック
                if bytes[i] == b'\'' {
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
