use chrono::Local;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::{AppError, Result};
use crate::language::LineProcessor;
use crate::stats::{FileStats, RunResult};

/// Run the file counting engine.
///
/// Returns a `RunResult` containing both successfully processed file statistics
/// and any errors encountered during processing.
///
/// # Errors
///
/// Returns an error only for critical failures (e.g., walk initialization).
/// Individual file processing errors are collected in `RunResult::errors`.
pub fn run(config: &Config) -> Result<RunResult> {
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
        // Strict mode: fail on first error
        let stats = iter
            .map(|item| process_file(item, config))
            .collect::<Result<Vec<_>>>()?;
        Ok(RunResult {
            stats,
            errors: Vec::new(),
        })
    } else {
        // Non-strict mode: collect errors alongside successful results
        let (results, errors): (Vec<_>, Vec<_>) = iter
            .map(|item| {
                let path = item.0.clone();
                process_file(item, config).map_err(|e| (path, e))
            })
            .partition(|r| r.is_ok());

        let stats: Vec<FileStats> = results.into_iter().map(|r| r.unwrap()).collect();
        let errors: Vec<(PathBuf, AppError)> = errors.into_iter().map(|r| r.unwrap_err()).collect();

        Ok(RunResult { stats, errors })
    }
}

fn process_file((path, meta): (PathBuf, std::fs::Metadata), config: &Config) -> Result<FileStats> {
    let size = meta.len();
    let mtime = meta.modified().ok().map(chrono::DateTime::<Local>::from);
    let mut stats = FileStats::new(path.clone());
    stats.size = size;
    stats.mtime = mtime;

    let file = File::open(&path).map_err(AppError::Io)?;
    let mut reader = BufReader::new(file);

    // Binary check (Initial buffer check)
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
    let mut lines = 0;
    let mut chars = 0;
    let mut words = 0;
    let mut sloc = 0;

    let count_words = config.count_words;
    let count_newlines = config.count_newlines_in_chars;

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let mut processor = crate::language::get_processor(ext, &config.filter.map_ext);

    let mut line_buf = Vec::new();

    loop {
        line_buf.clear();
        match reader.read_until(b'\n', &mut line_buf) {
            Ok(0) => break,
            Ok(_) => {
                lines += 1;

                // Use lossy conversion to support non-UTF8 text files (mostly)
                let cow = String::from_utf8_lossy(&line_buf);
                let line_str = &cow;

                // Single-pass processing for chars, words, and SLOC
                let l_stats = processor.process_line_stats(line_str, count_words, count_newlines);

                chars += l_stats.chars;
                sloc += l_stats.sloc;

                if count_words {
                    words += l_stats.words;
                }
            }
            Err(e) => return Err(AppError::Io(e)),
        }
    }

    stats.lines = lines;
    stats.chars = chars;
    if count_words {
        stats.words = Some(words);
    }
    stats.sloc = Some(sloc);

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
        let chunk_chars = bytecount::num_chars(buf);
        if count_newlines {
            chars += chunk_chars;
        } else {
            let lf_count = bytecount::count(buf, b'\n');
            let cr_count = bytecount::count(buf, b'\r');
            chars += chunk_chars;
            chars -= lf_count;
            if cr_count > 0 {
                chars -= cr_count;
            }
        }

        // Count words with UTF-8 validation
        if count_words {
            // Use lossy conversion to support non-UTF8 text and handle split multi-byte seq
            let cow = String::from_utf8_lossy(buf);
            words += cow.split_whitespace().count();
        }

        let len = buf.len();
        reader.consume(len);
    }

    // 末尾に改行がない場合の行カウント補正
    if let Some(b) = last_byte
        && b != b'\n'
    {
        lines += 1;
    }

    stats.lines = lines;
    stats.chars = chars;
    if count_words {
        stats.words = Some(words);
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
}
