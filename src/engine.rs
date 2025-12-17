use chrono::Local;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use crate::config::Config;
use crate::error::{AppError, Result};
use crate::language::{LineProcessor, SlocProcessor};
use crate::stats::FileStats; // Removed duplicate imports or clean up if needed

pub fn run(config: &Config) -> Result<Vec<FileStats>> {
    let (tx, rx) = crossbeam_channel::bounded(1024);

    let walk_cfg = config.walk.clone();
    let filter_cfg = config.filter.clone();
    std::thread::spawn(move || {
        if let Err(e) = crate::filesystem::walk_parallel(&walk_cfg, &filter_cfg, &tx) {
            eprintln!("Walk error: {e}");
        }
    });

    let iter = rx.into_iter().par_bridge();

    if config.strict {
        iter.map(|item| process_file(item, config))
            .collect::<Result<Vec<_>>>()
    } else {
        Ok(iter
            .filter_map(|item| {
                let path = item.0.clone();
                match process_file(item, config) {
                    Ok(stats) => Some(stats),
                    Err(e) => {
                        // TODO: Verbose logging?
                        eprintln!("Error processing {}: {}", path.display(), e);
                        None
                    }
                }
            })
            .collect())
    }
}

fn process_file((path, meta): (PathBuf, std::fs::Metadata), config: &Config) -> Result<FileStats> {
    // check_filters moved to walk_parallel

    let size = meta.len();
    let mtime = meta.modified().ok().map(chrono::DateTime::<Local>::from);
    let mut stats = FileStats::new(path.clone());
    stats.size = size;
    stats.mtime = mtime;

    let file = File::open(&path).map_err(AppError::Io)?;
    let mut reader = BufReader::new(file);

    // Binary check
    {
        let buffer = reader.fill_buf().map_err(AppError::Io)?;
        if buffer.is_empty() {
            return Ok(stats);
        }
        if buffer.contains(&0) {
            stats.is_binary = true;
            return Ok(stats);
        }
    }
    // No need to seek, fill_buf doesn't consume the buffer.

    let content_stats = process_content(&mut reader, config, &path)?;
    stats.lines = content_stats.lines;
    stats.chars = content_stats.chars;
    stats.words = content_stats.words;
    stats.sloc = content_stats.sloc;
    stats.is_binary = content_stats.is_binary; // In case interpret_content found binary

    Ok(stats)
}

fn process_content<R: BufRead>(
    reader: &mut R,
    config: &Config,
    path: &std::path::Path,
) -> Result<FileStats> {
    let mut stats = FileStats::new(path.to_path_buf());
    let mut lines = 0;
    let mut chars = 0;
    let mut words = 0;
    let mut sloc = 0;

    let count_words = config.count_words;
    let count_sloc = config.count_sloc;

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut processor = if count_sloc {
        Some(SlocProcessor::from_extension(ext))
    } else {
        None
    };

    // If we need SLOC, we need line-by-line processing.
    // If we just need lines/chars/words, we can stream.
    // Currently, for simplicity and performance balance, we use read_until for SLOC
    // and fill_buf for others?
    // Actually, read_until reusing a Vec<u8> is consistently good enough for avoiding allocation,
    // provided we check UTF-8 properly.

    // However, the critique specifically asked for fill_buf for "just counting",
    // so let's separate the paths.

    if count_sloc {
        // Line-based processing needed
        let mut line_buf = Vec::new(); // Reusable buffer
        loop {
            line_buf.clear();
            match reader.read_until(b'\n', &mut line_buf) {
                Ok(0) => break,
                Ok(_) => {
                    lines += 1;

                    // Decode for SLOC and other stats
                    // We try to decode as UTF-8. If fails, we mark as binary?
                    match std::str::from_utf8(&line_buf) {
                        Ok(line_str) => {
                            // Stats
                            if config.count_newlines_in_chars {
                                chars += line_str.chars().count();
                                // Can use bytecount::num_chars(&line_buf) but line_str is already checked
                            } else {
                                let mut c = line_str.chars().count();
                                if line_str.ends_with("\r\n") {
                                    c = c.saturating_sub(2);
                                } else if line_str.ends_with('\n') {
                                    c = c.saturating_sub(1);
                                }
                                chars += c;
                            }

                            if count_words {
                                words += line_str.split_whitespace().count();
                            }

                            if let Some(proc) = &mut processor {
                                sloc += proc.process_line(line_str);
                            }
                        }
                        Err(_) => {
                            // Invalid UTF-8: likely binary
                            stats.is_binary = true;
                            // We can still count lines/chars based on bytes if we want,
                            // or just return binary status.
                            // Current logic: return strict binary result
                            stats.lines = lines;
                            stats.chars = chars;
                            return Ok(stats);
                        }
                    }
                }
                Err(e) => return Err(AppError::Io(e)),
            }
        }
    } else {
        // Streaming fast path
        let mut in_word = false;
        let mut last_byte: Option<u8> = None;
        loop {
            let buf = reader.fill_buf().map_err(AppError::Io)?;
            if buf.is_empty() {
                break;
            }

            if let Some(&b) = buf.last() {
                last_byte = Some(b);
            }

            // Count lines
            lines += bytecount::count(buf, b'\n');

            // Count chars
            // bytecount::num_chars counts valid leading UTF-8 bytes.
            let chunk_chars = bytecount::num_chars(buf);
            if config.count_newlines_in_chars {
                chars += chunk_chars;
            } else {
                // We need to subtract newlines.
                // Note: num_chars counts \n as 1 char.
                // If CRLF, \r is 1 char, \n is 1 char.
                // Wait, logic says we want to exclude newline chars from count.
                // Simple approach: chars += chunk_chars - count(buf, \n).
                // What about \r? simple count subtraction?
                // If strict CRLF handling is needed, we need to check byte pairs.
                // For perf, maybe just subtracting '\n' count is approximation used in bytecount optimization?
                // The original code handled ends_with logic.
                // For streaming: `bytecount::count(buf, b'\n')` gives LF count.
                // `bytecount::count(buf, b'\r')` gives CR count.
                // If we want "text content length", subtracting these is decent.
                // Let's stick to subtraction of LF for now to match rough behavior,
                // or if strict, we iterate.
                // Critique said "use bytecount".

                let lf_count = bytecount::count(buf, b'\n');
                let cr_count = bytecount::count(buf, b'\r'); // approximate CRLF/CR
                chars += chunk_chars;
                chars -= lf_count;
                if cr_count > 0 {
                    // Check if the file is CRLF?
                    // To be safe, just subtract CR too.
                    chars -= cr_count;
                }
            }

            if count_words {
                // Simple word count state machine on bytes
                // Note: this assumes ASCII whitespace for word boundaries or UTF-8?
                // split_whitespace relies on Unicode Property White_Space.
                // bytecount doesn't have words.
                // If we want speed, maybe assuming ASCII whitespace is acceptable?
                // Or decoding chunk?
                // Decoding chunk is safe for num_chars (stateful).
                // For words:
                for &b in buf {
                    let is_whitespace = b.is_ascii_whitespace(); // optimization approximation
                    if in_word && is_whitespace {
                        words += 1;
                        in_word = false;
                    } else if !in_word && !is_whitespace {
                        in_word = true;
                    }
                }
                // Edge case: end of buffer in middle of word. `in_word` preserves state.
                // If buffer ends in word, `words` not incremented yet (wait for next space or EOF).
            }

            let len = buf.len();
            reader.consume(len);
        }
        if in_word {
            words += 1;
        }
        // If file ends without newline, count that line
        if let Some(b) = last_byte {
            if b != b'\n' {
                lines += 1;
            }
        }
    }

    stats.lines = lines;
    stats.chars = chars;
    if count_words {
        stats.words = Some(words);
    }
    if count_sloc {
        stats.sloc = Some(sloc);
    }

    Ok(stats)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_count_chars_trailing_spaces() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "abc \nde \r\nfg ").unwrap();
        let path = file.path().to_path_buf();

        let mut config = Config::default();
        config.count_newlines_in_chars = false;
        config.filter.allow_ext.clear();

        // process_file is private, but testing inner module has access?
        // Wait, tests module is inside engine.rs, so it has access to private items of parent module `super::*`.
        let meta = std::fs::metadata(&path).unwrap();
        let stats = process_file((path, meta), &config).unwrap();

        assert_eq!(stats.chars, 10);
        assert_eq!(stats.lines, 3);
    }
}
