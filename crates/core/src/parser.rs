use crate::config::AnalysisConfig;
use crate::language::get_processor;
use crate::stats::AnalysisResult;

/// Count lines/chars/words/sloc in a byte slice.
///
/// This is the core entry point for the library.
/// It mimics the logic from the original `process_content_sloc` but works on in-memory bytes.
/// It also handles binary detection (simplistic check).
#[must_use]
pub fn count_bytes(input: &[u8], extension: &str, config: &AnalysisConfig) -> AnalysisResult {
    let mut stats = AnalysisResult::new();

    // 1. Binary check
    if is_binary(input) {
        stats.is_binary = true;
        // If binary, we usually stop or simplistic count?
        // Original logic: if binary, return stats with is_binary=true.
        // Usually line counting binary files is not useful.
        // But for compatibility let's just mark it and maybe return 0 lines or simple line count?
        // Original `process_file` returns early if binary.
        return stats;
    }

    // 2. Convert to lossy string
    // We use lossy to handle potential non-UTF8 text files gracefully.
    let text = crate::language::string_utils::from_utf8_lossy(input);

    // 3. Process
    let mut processor = get_processor(extension, &config.map_ext);

    let mut lines = 0;
    let mut chars = 0;
    let mut words = 0;
    let mut sloc = 0;

    for line in text.lines() {
        lines += 1;

        let l_stats =
            processor.process_line_stats(line, config.count_words, config.count_newlines_in_chars);

        chars += l_stats.chars;
        sloc += l_stats.sloc;
        if config.count_words {
            words += l_stats.words;
        }
    }

    // Handle trailing newline case if necessary?
    // text.lines() ignores the final newline.
    // Note: The original implementation in `process_content_sloc` used `read_until(b'\n')`
    // and then processed the line.
    // If the input ends with a newline, `lines()` will NOT yield an empty string for the part after it.
    // However, if the file is just "a\n", `lines()` yields "a". Count is 1.
    // If the file is "a", `lines()` yields "a". Count is 1.
    // So lines() is mostly correct for line counting.

    stats.lines = lines;
    stats.chars = chars;
    if config.count_words {
        stats.words = Some(words);
    }
    stats.sloc = Some(sloc);

    stats
}

fn is_binary(input: &[u8]) -> bool {
    // Determine if content is binary by checking for NUL bytes in the first 8KB
    // Original used 1024 bytes buffer from BufReader
    let len = input.len().min(8 * 1024);
    input[..len].contains(&0)
}
