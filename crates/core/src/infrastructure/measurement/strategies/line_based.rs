use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::domain::{
    config::Config,
    model::{FileMeta, FileStats},
};

/// Measure a file incrementally by iterating over its lines.
pub fn measure_by_lines(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let (mut lines, mut chars, mut words) = (0, 0, 0);
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader.read_line(&mut line).ok()?;
        if n == 0 {
            break;
        }
        if line.ends_with('\n') {
            line.pop();
            if line.ends_with('\r') {
                line.pop();
            }
        }
        lines += 1;
        chars += line.chars().count();
        if config.words {
            words += line.split_whitespace().count();
        }
    }
    Some(FileStats::new(path.to_path_buf(), lines, chars, config.words.then_some(words), meta))
}
