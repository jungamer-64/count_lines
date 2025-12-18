
use proptest::prelude::*;
use count_lines_core::parser::count_bytes;
use count_lines_core::config::AnalysisConfig;

proptest! {
    #[test]
    fn test_line_count_never_exceeds_byte_count(
        content in "[\\x00-\\x7F]{0,1000}"
    ) {
        // Line count should generally not exceed byte count.
        // Exception: Check empty file.
        // "" -> 0 bytes, 0 lines.
        // "a" -> 1 byte, 1 line.
        let stats = count_bytes(content.as_bytes(), "txt", &AnalysisConfig::default());
        let len = content.len();
        if len > 0 {
            assert!(stats.lines <= len);
        } else {
            assert_eq!(stats.lines, 0);
        }
    }
    
    #[test]
    fn test_char_count_consistent_with_unicode(
        content in "\\PC{0,500}"
    ) {
        let config = AnalysisConfig {
            count_newlines_in_chars: true,
            ..AnalysisConfig::default()
        };
        let stats = count_bytes(content.as_bytes(), "txt", &config);
        let expected = content.chars().count();
        prop_assert_eq!(stats.chars, expected);
    }
}
