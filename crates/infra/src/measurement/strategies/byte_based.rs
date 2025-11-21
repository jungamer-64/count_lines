use std::path::Path;

use crate::persistence::FileReader;
use count_lines_domain::{
    config::Config,
    model::{FileMeta, FileStatsV2},
    value_objects::{
        CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
    },
};

/// Measure a file by reading it into memory and counting bytes/lines/words.
pub fn measure_entire_file(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStatsV2> {
    let buf = FileReader::read_to_end(path).ok()?;
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

    let builder = FileStatsV2::builder(FilePath::new(path.to_path_buf()))
        .lines(LineCount::new(lines))
        .chars(CharCount::new(chars))
        .words(words.map(WordCount::new))
        .size(FileSize::new(meta.size))
        .mtime(meta.mtime.map(ModificationTime::new))
        .ext(FileExtension::new(meta.ext.clone()))
        .name(FileName::new(meta.name.clone()));

    Some(builder.build())
}
