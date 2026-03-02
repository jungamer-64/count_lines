// crates/engine/src/processor.rs
use crate::config::Config;
use crate::error::{EngineError, Result};
use crate::stats::FileStats;
use count_lines_core::language::LineProcessor;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub fn process_file(
    (path, meta): (PathBuf, std::fs::Metadata),
    config: &Config,
) -> Result<FileStats> {
    let size = meta.len();
    let mtime = meta
        .modified()
        .ok()
        .map(chrono::DateTime::<chrono::Local>::from);
    let mut stats = FileStats::new(path.clone());
    stats.size = size;
    stats.mtime = mtime;

    let file = File::open(&path).map_err(|e| EngineError::FileRead {
        path: path.clone(),
        source: e,
    })?;
    let mut reader = BufReader::new(file);

    // Binary check (Initial buffer check)
    {
        let buffer = reader.fill_buf().map_err(|e| EngineError::FileRead {
            path: path.clone(),
            source: e,
        })?;
        if buffer.is_empty() {
            return Ok(stats);
        }
        if buffer.contains(&0) {
            stats.is_binary = true;
            return Ok(stats);
        }
    }

    let content_stats = process_content(&mut reader, config, &path)?;
    stats.lines = content_stats.lines;
    stats.chars = content_stats.chars;
    stats.words = content_stats.words;
    stats.sloc = content_stats.sloc;

    // ストリーミング処理中にバイナリ判定された場合に対応
    if content_stats.is_binary {
        stats.is_binary = true;
    }

    Ok(stats)
}

/// コンテンツ処理のメインディスパッチャ
fn process_content<R: BufRead>(reader: &mut R, config: &Config, path: &Path) -> Result<FileStats> {
    if config.count_sloc {
        process_content_sloc(reader, config, path)
    } else {
        process_content_streaming(reader, config, path)
    }
}

/// SLOCカウント用の行ベース処理
fn process_content_sloc<R: BufRead>(
    reader: &mut R,
    config: &Config,
    path: &Path,
) -> Result<FileStats> {
    let mut stats = FileStats::new(path.to_path_buf());
    stats.sloc = Some(0);

    let count_words = config.count_words;
    let count_newlines = config.count_newlines_in_chars;

    if count_words {
        stats.words = Some(0);
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut processor = count_lines_core::language::get_processor(ext, &config.filter.map_ext);

    let mut line_buf = Vec::new();

    loop {
        line_buf.clear();
        match reader.read_until(b'\n', &mut line_buf) {
            Ok(0) => break,
            Ok(_) => {
                stats.lines += 1;

                // Use lossy conversion to support non-UTF8 text files (mostly)
                let cow = String::from_utf8_lossy(&line_buf);
                let line_str = &cow;

                // Single-pass processing for chars, words, and SLOC
                let l_stats = processor.process_line_stats(line_str, count_words, count_newlines);

                stats.chars += l_stats.chars;
                *stats.sloc.as_mut().unwrap() += l_stats.sloc;

                if let Some(w) = stats.words.as_mut() {
                    *w += l_stats.words;
                }
            }
            Err(e) => {
                return Err(EngineError::FileRead {
                    path: path.to_path_buf(),
                    source: e,
                });
            }
        }
    }

    Ok(stats)
}

/// 高速処理用のストリーミング処理
///
/// Binary detection strategy:
/// 1. Initial check: Look for NUL bytes in first buffer (done in `process_file`)
/// 2. Streaming check: Detect invalid UTF-8 during word counting
///
/// Word counting:
/// - When word counting is enabled, we validate UTF-8 and use Unicode-aware splitting
/// - If invalid UTF-8 is detected, the file is marked as binary
/// - When word counting is disabled, we use fast byte counting without UTF-8 validation
fn process_content_streaming<R: BufRead>(
    reader: &mut R,
    config: &Config,
    path: &Path,
) -> Result<FileStats> {
    let mut stats = FileStats::new(path.to_path_buf());
    let mut lines = 0;
    let mut chars = 0;
    let mut words = 0;

    let count_words = config.count_words;
    let count_newlines = config.count_newlines_in_chars;

    let mut prev_ended_with_non_whitespace = false;
    let mut partial_utf8 = Vec::new();

    let mut ends_with_lf = true;


    loop {
        let buf = reader.fill_buf().map_err(|e| EngineError::FileRead {
            path: path.to_path_buf(),
            source: e,
        })?;
        
        if buf.is_empty() {
            if !partial_utf8.is_empty() {
                // Handle remaining bytes if any
                let cow = String::from_utf8_lossy(&partial_utf8);
                if count_words {
                    words += count_words_in_segment(&cow, &mut prev_ended_with_non_whitespace);
                }
                
                for c in cow.chars() {
                    if count_newlines || (c != '\n' && c != '\r') {
                        chars += 1;
                    }
                }
                ends_with_lf = cow.as_bytes().last() == Some(&b'\n');
            }
            break;
        }

        lines += bytecount::count(buf, b'\n');

        // Optimized path for UTF-8 and words
        if count_words || !count_newlines || true { // true because we always need chars currently or binary check
            let current_data = if partial_utf8.is_empty() {
                buf
            } else {
                partial_utf8.extend_from_slice(buf);
                &partial_utf8
            };

            let (valid_part, invalid_part) = split_at_valid_utf8(current_data);
            
            if !valid_part.is_empty() {
                // Safety: split_at_valid_utf8 ensures valid_part is valid UTF-8
                let s = unsafe { std::str::from_utf8_unchecked(valid_part) };
                
                if count_words {
                    words += count_words_in_segment(s, &mut prev_ended_with_non_whitespace);
                }
                
                // Count actual Unicode characters
                for c in s.chars() {
                    if count_newlines || (c != '\n' && c != '\r') {
                        chars += 1;
                    }
                }
                
                ends_with_lf = valid_part.last() == Some(&b'\n');
            }

            if !invalid_part.is_empty() {
                // If we have an invalid part and it's not a split multi-byte char, it's binary
                if invalid_part.len() >= 4 || !is_potentially_split_utf8(invalid_part) {
                    stats.is_binary = true;
                }
                
                // Keep for next chunk if it looks like a split UTF-8 character
                let mut new_partial = Vec::with_capacity(invalid_part.len());
                new_partial.extend_from_slice(invalid_part);
                partial_utf8 = new_partial;
            } else {
                partial_utf8.clear();
            }
        }

        let len = buf.len();
        reader.consume(len);
    }

    stats.lines = lines;
    // 末尾に改行がない場合の行カウント補正
    if !ends_with_lf && (chars > 0 || words > 0 || !partial_utf8.is_empty()) {
        stats.lines += 1;
    }
    
    stats.chars = chars;
    if count_words {
        stats.words = Some(words);
    }

    Ok(stats)
}

fn split_at_valid_utf8(buf: &[u8]) -> (&[u8], &[u8]) {
    match std::str::from_utf8(buf) {
        Ok(_) => (buf, &[]),
        Err(e) => {
            let valid_len = e.valid_up_to();
            buf.split_at(valid_len)
        }
    }
}

fn is_potentially_split_utf8(buf: &[u8]) -> bool {
    let len = buf.len();
    if len == 0 || len >= 4 {
        return false;
    }
    // Look for a start byte
    for i in (0..len).rev() {
        let b = buf[i];
        if b & 0b1100_0000 == 0b1100_0000 {
            // Found a start byte. Is it potentially asking for more bytes?
            let expected = if b & 0b1110_0000 == 0b1100_0000 {
                2
            } else if b & 0b1111_0000 == 0b1110_0000 {
                3
            } else if b & 0b1111_1000 == 0b1111_0000 {
                4
            } else {
                0
            };
            return expected > (len - i);
        }
        if b & 0b1000_0000 == 0 {
            // ASCII byte in the middle of invalid part? Shouldn't happen with split_at_valid_utf8 logic
            return false;
        }
    }
    true
}

fn count_words_in_segment(s: &str, prev_ended_with_non_whitespace: &mut bool) -> usize {
    let mut chunk_words = s.split_whitespace().count();
    if *prev_ended_with_non_whitespace {
        if let Some(first_char) = s.chars().next() {
            if !first_char.is_whitespace() {
                chunk_words = chunk_words.saturating_sub(1);
            }
        }
    }
    *prev_ended_with_non_whitespace = if let Some(last_char) = s.chars().last() {
        !last_char.is_whitespace()
    } else {
        false
    };
    chunk_words
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

        let config = Config {
            count_newlines_in_chars: false,
            filter: crate::config::FilterConfig {
                allow_ext: vec![],
                ..crate::config::FilterConfig::default()
            },
            ..Config::default()
        };

        let meta = std::fs::metadata(&path).unwrap();
        let stats = process_file((path, meta), &config).unwrap();

        assert_eq!(stats.chars, 10);
        assert_eq!(stats.lines, 3);
    }

    #[test]
    fn test_process_content_streaming_split_word() {
        let mut config = Config::default();
        config.count_sloc = false; // Force streaming mode
        config.count_words = true;
        
        // "hello wor" (9 bytes) + "ld" (2 bytes) = "hello world" (1 word)
        // If split at chunk boundary, it should still be 1 word.
        
        let path = PathBuf::from("test.txt");
        
        // Mock a reader that returns in chunks
        struct ChunkReader {
            chunks: Vec<Vec<u8>>,
            current: usize,
        }
        impl std::io::Read for ChunkReader {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if self.current >= self.chunks.len() {
                    return Ok(0);
                }
                let chunk = &self.chunks[self.current];
                let len = chunk.len().min(buf.len());
                buf[..len].copy_from_slice(&chunk[..len]);
                self.current += 1;
                Ok(len)
            }
        }
        impl BufRead for ChunkReader {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                if self.current >= self.chunks.len() {
                    return Ok(&[]);
                }
                Ok(&self.chunks[self.current])
            }
            fn consume(&mut self, _amt: usize) {
                self.current += 1;
            }
        }

        let mut reader = ChunkReader {
            chunks: vec![b"hello wor".to_vec(), b"ld".to_vec()],
            current: 0,
        };

        let stats = process_content_streaming(&mut reader, &config, &path).unwrap();
        assert_eq!(stats.words, Some(2)); // WAIT, "hello wor" + "ld" -> "hello", "wor" + "ld" -> 2 words total if split
        // Actually, split_whitespace() on "hello wor" is 2 words: ["hello", "wor"]
        // split_whitespace() on "ld" is 1 word: ["ld"]
        // Total 2 words.
        
        // Let's try something simpler: "hello" split into "he" and "llo"
        let mut reader2 = ChunkReader {
            chunks: vec![b"he".to_vec(), b"llo".to_vec()],
            current: 0,
        };
        let stats2 = process_content_streaming(&mut reader2, &config, &path).unwrap();
        assert_eq!(stats2.words, Some(1));

        // Test with whitespace in between: "he " and "llo"
        let mut reader3 = ChunkReader {
            chunks: vec![b"he ".to_vec(), b"llo".to_vec()],
            current: 0,
        };
        let stats3 = process_content_streaming(&mut reader3, &config, &path).unwrap();
        assert_eq!(stats3.words, Some(2));
    }
}
