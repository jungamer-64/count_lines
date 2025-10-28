
use crate::{
    domain::{
        config::Config,
        model::{FileMeta, FileStats},
    },
    infrastructure::persistence::FileReader,
};
use std::{io::BufRead, path::Path};

/// 行単位でファイルを計測
pub fn measure_by_lines(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
    let reader = FileReader::open_buffered(path).ok()?;

    let mut line_count = 0;
    let mut char_count = 0;
    let mut word_count = 0;

    for line_result in reader.lines() {
        let line = line_result.ok()?;
        
        line_count += 1;
        char_count += line.chars().count();
        
        if config.words {
            word_count += line.split_whitespace().count();
        }
    }

    let words = config.words.then_some(word_count);
    Some(FileStats::new(path.to_path_buf(), line_count, char_count, words, meta))
}