// crates/core/src/language/processors/vhdl_style.rs
//! VHDLのコメント処理
//!
//! `--` 行コメントのみをサポートします。
//! VHDL-2008でブロックコメントが追加されましたが、多くの処理系が未対応なので行コメントのみ。

/// VHDL スタイル (-- のみ) の処理
///
/// VHDL:
/// - `--` 以降が行コメント
/// - VHDL-2008ではブロックコメントがあるが、多くの処理系が未対応なので行コメントのみ
#[cfg(test)]
fn process_vhdl_style(line: &str, count: &mut usize) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vhdl_line_comment() {
        let mut count = 0;
        process_vhdl_style("-- VHDL comment", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_vhdl_code() {
        let mut count = 0;
        process_vhdl_style("signal clk : std_logic;", &mut count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_vhdl_inline_comment() {
        let mut count = 0;
        process_vhdl_style("signal rst : std_logic; -- reset signal", &mut count);
        process_vhdl_style("signal data : std_logic_vector(7 downto 0);", &mut count);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_vhdl_entity() {
        let mut count = 0;
        process_vhdl_style("-- Entity declaration", &mut count);
        process_vhdl_style("entity counter is", &mut count);
        process_vhdl_style("port (", &mut count);
        process_vhdl_style("    clk : in std_logic; -- clock input", &mut count);
        process_vhdl_style("    -- rst : in std_logic;", &mut count);
        process_vhdl_style("    count : out std_logic_vector(7 downto 0)", &mut count);
        process_vhdl_style(");", &mut count);
        process_vhdl_style("end entity;", &mut count);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_vhdl_architecture() {
        let mut count = 0;
        process_vhdl_style("architecture behavioral of counter is", &mut count);
        process_vhdl_style("begin", &mut count);
        process_vhdl_style("    -- Process block", &mut count);
        process_vhdl_style("    process(clk)", &mut count);
        process_vhdl_style("    begin", &mut count);
        process_vhdl_style("        if rising_edge(clk) then", &mut count);
        process_vhdl_style("            count <= count + 1; -- increment", &mut count);
        process_vhdl_style("        end if;", &mut count);
        process_vhdl_style("    end process;", &mut count);
        process_vhdl_style("end architecture;", &mut count);
        assert_eq!(count, 9);
    }

    #[test]
    fn test_vhdl_empty_line() {
        let mut count = 0;
        process_vhdl_style("", &mut count);
        assert_eq!(count, 1); // Note: empty line handling is done at caller level
    }

    #[test]
    fn test_vhdl_comment_only_dashes() {
        let mut count = 0;
        process_vhdl_style("--", &mut count);
        assert_eq!(count, 0);
    }

    #[test]
    fn test_vhdl_library_use() {
        let mut count = 0;
        process_vhdl_style("library ieee;", &mut count);
        process_vhdl_style("use ieee.std_logic_1164.all;", &mut count);
        process_vhdl_style("use ieee.numeric_std.all; -- for arithmetic", &mut count);
        assert_eq!(count, 3);
    }
}
