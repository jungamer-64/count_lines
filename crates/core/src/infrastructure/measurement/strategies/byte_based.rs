use crate::domain::config::Config;
use crate::domain::model::{FileMeta, FileStats};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Measure a file by reading it into memory and counting bytes/lines/words.
pub fn measure_entire_file(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
    let mut file = File::open(path).ok()?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).ok()?;
    if config.text_only && buf.contains(&0) {
        return None;
    }
    let content = String::from_utf8_lossy(&buf);
    let bytes = content.as_bytes();
    let newline_count = bytecount::count(bytes, b'\n');
    let lines = if bytes.is_empty() {
        0
    } else if bytes.last() == Some(&b'\n') {
        newline_count
    } else {
        newline_count + 1
    };
    let chars = content.chars().count();
    let words = config.words.then(|| content.split_whitespace().count());
    Some(FileStats::new(
        path.to_path_buf(),
        lines,
        chars,
        words,
        meta,
    ))
}
