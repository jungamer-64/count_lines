// crates/core/src/counter.rs
use crate::config::AnalysisConfig;
use crate::language::get_processor;
use crate::stats::AnalysisResult;

/// Count lines/chars/words/sloc in a byte slice.
///
/// This is the core entry point for the library.
/// Processes in-memory bytes with binary detection and per-line SLOC analysis.
#[must_use]
pub fn count_bytes(input: &[u8], extension: &str, config: &AnalysisConfig) -> AnalysisResult {
    let mut stats = AnalysisResult::new();

    // Binary check: skip counting for binary files
    if is_binary(input) {
        stats.is_binary = true;
        return stats;
    }

    // 2. Process line by line
    let mut processor = get_processor(extension, &config.map_ext);

    let mut lines = 0;
    let mut chars = 0;
    let mut words = 0;
    let mut sloc = 0;

    // Use split_inclusive on bytes to avoid allocating a full String for the file
    // if it contains invalid UTF-8.
    for line_bytes in input.split_inclusive(|&b| b == b'\n') {
        lines += 1;

        // Convert line to lossy string (zero-copy if valid UTF-8)
        let line = crate::language::string_utils::from_utf8_lossy(line_bytes);

        let l_stats =
            processor.process_line_stats(&line, config.count_words, config.count_newlines_in_chars);

        chars += l_stats.chars;
        sloc += l_stats.sloc;
        if config.count_words {
            words += l_stats.words;
        }
    }


    stats.lines = lines;
    stats.chars = chars;
    if config.count_words {
        stats.words = Some(words);
    }
    stats.sloc = Some(sloc);

    stats
}

fn is_binary(input: &[u8]) -> bool {
    // Check for NUL bytes in the first 8KB to detect binary content
    let len = input.len().min(8 * 1024);
    input[..len].contains(&0)
}
