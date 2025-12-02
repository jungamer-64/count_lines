// crates/infra/src/measurement/strategies/sloc_counter/processors/assembly_style.rs
//! アセンブリ言語のコメント処理
//!
//! Intel形式 (NASM/MASM)、AT&T形式 (GAS)、VHDL を処理します。

/// Assembly (NASM/MASM) スタイル (; のみ) の処理
///
/// Intel形式アセンブリ (NASM, MASM等):
/// - `;` 以降が行コメント
pub fn process_assembly_style(line: &str, count: &mut usize) {
    // ; から始まる場合はコメント行
    if line.starts_with(';') {
        return;
    }

    // 行中に ; がある場合、その前にコードがあればカウント
    if let Some(pos) = line.find(';') {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }

    *count += 1;
}

/// GAS (GNU Assembler) スタイル (# と /* */) の処理
///
/// AT&T形式アセンブリ (GAS, ARM等):
/// - `#` 以降が行コメント (プリプロセッサも)
/// - `/* */` ブロックコメント
/// - 一部のGASでは `//` も使用可能だが、`#` が主流
pub fn process_gas_assembly_style(
    line: &str,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    if *in_block_comment {
        if let Some(pos) = line.find("*/") {
            *in_block_comment = false;
            let rest = &line[pos + 2..];
            if !rest.trim().is_empty() {
                // 残りを再帰処理
                process_gas_assembly_style(rest, in_block_comment, count);
            }
        }
        return;
    }

    // # 行コメント
    if line.starts_with('#') || line.starts_with('@') {
        // @ も ARM GAS でコメント
        return;
    }

    // 行中の # コメント
    if let Some(hash_pos) = line.find('#') {
        let before = &line[..hash_pos];

        // # の前に /* があるかチェック
        if let Some(block_start) = before.find("/*") {
            // ブロックコメントが先
            process_gas_block_comment(line, block_start, in_block_comment, count);
            return;
        }

        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }

    // /* ブロックコメント
    if let Some(block_start) = line.find("/*") {
        process_gas_block_comment(line, block_start, in_block_comment, count);
        return;
    }

    *count += 1;
}

/// GAS ブロックコメント処理のヘルパー
fn process_gas_block_comment(
    line: &str,
    block_start: usize,
    in_block_comment: &mut bool,
    count: &mut usize,
) {
    let before = &line[..block_start];
    let has_code_before = !before.trim().is_empty();

    let after_start = &line[block_start + 2..];
    if let Some(end_offset) = after_start.find("*/") {
        // 同じ行で閉じる
        let after = &after_start[end_offset + 2..];
        if has_code_before {
            *count += 1;
        } else if !after.trim().is_empty() {
            process_gas_assembly_style(after, in_block_comment, count);
        }
    } else {
        *in_block_comment = true;
        if has_code_before {
            *count += 1;
        }
    }
}

/// VHDL スタイル (-- のみ) の処理
///
/// VHDL:
/// - `--` 以降が行コメント
/// - VHDL-2008ではブロックコメントがあるが、多くの処理系が未対応なので行コメントのみ
pub fn process_vhdl_style(line: &str, count: &mut usize) {
    // -- から始まる場合はコメント行
    if line.starts_with("--") {
        return;
    }

    // 行中に -- がある場合、その前にコードがあればカウント
    if let Some(pos) = line.find("--") {
        let before = &line[..pos];
        if !before.trim().is_empty() {
            *count += 1;
        }
        return;
    }

    *count += 1;
}
