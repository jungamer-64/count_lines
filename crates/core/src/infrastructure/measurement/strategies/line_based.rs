use std::{io::BufRead, path::Path};

use crate::{
    domain::{
        config::Config,
        model::{FileMeta, FileStatsV2},
        value_objects::{
            CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
        },
    },
    infrastructure::persistence::FileReader,
};

/// 行単位でファイルを計測
pub fn measure_by_lines(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStatsV2> {
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

    let builder = FileStatsV2::builder(FilePath::new(path.to_path_buf()))
        .lines(LineCount::new(line_count))
        .chars(CharCount::new(char_count))
        .words(config.words.then_some(WordCount::new(word_count)))
        .size(FileSize::new(meta.size))
        .mtime(meta.mtime.map(ModificationTime::new))
        .ext(FileExtension::new(meta.ext.clone()))
        .name(FileName::new(meta.name.clone()));

    Some(builder.build())
}
