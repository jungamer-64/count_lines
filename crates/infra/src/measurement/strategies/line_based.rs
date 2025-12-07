use std::{io::BufRead, path::Path};

use unicode_segmentation::UnicodeSegmentation;

use super::sloc_counter::SlocCounter;
use crate::persistence::FileReader;
use count_lines_domain::{
    config::Config,
    model::{FileMeta, FileStatsV2},
    value_objects::{
        CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, SlocCount,
        WordCount,
    },
};

/// 行単位でファイルを計測
pub fn measure_by_lines(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStatsV2> {
    let mut reader = FileReader::open_buffered(path).ok()?;

    let mut line_count = 0;
    let mut char_count = 0;
    let mut word_count = 0;

    // SLOC: 言語対応のコメント除外カウンター
    let mut sloc_counter = SlocCounter::new(&meta.ext);

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => return None,
        }
        if config.text_only && line.contains('\0') {
            return None;
        }
        line_count += 1;
        
        // Grapheme counting requires full string slice, so we handle newlines carefully.
        let mut without_newline = line.as_str();
        if without_newline.ends_with('\n') {
            without_newline = &without_newline[..without_newline.len() - 1];
            if without_newline.ends_with('\r') {
                without_newline = &without_newline[..without_newline.len() - 1];
            }
        }

        let base_chars = without_newline.graphemes(true).count();
        
        if config.count_newlines_in_chars {
            // Count newline characters separately as simple chars since they are control chars
            // and we want to preserve the behavior of "chars" (which usually means scalar values for control chars)
            // OR we can just count the graphemes of the full line.
            // However, `graphemes` on "\r\n" might return 1 grapheme (CRLF is a grapheme cluster).
            // If the user wants "character count" including newlines, they usually expect CRLF to be 2 chars or 1 char depending on definition.
            // The previous implementation used `chars().count()`, so CRLF was 2 chars.
            // `graphemes(true)` treats CRLF as 1 extended grapheme cluster.
            // To be safe and consistent with "visual" counting, we count the full line graphemes.
            char_count += line.graphemes(true).count();
        } else {
            char_count += base_chars;
        }

        if config.words {
            word_count += without_newline.split_whitespace().count();
        }

        // SLOC: 言語対応のコメント除外処理
        if config.sloc {
            sloc_counter.process_line(without_newline);
        }
    }

    let builder = FileStatsV2::builder(FilePath::new(path.to_path_buf()))
        .lines(LineCount::new(line_count))
        .chars(CharCount::new(char_count))
        .words(config.words.then_some(WordCount::new(word_count)))
        .sloc(config.sloc.then_some(SlocCount::new(sloc_counter.count())))
        .size(FileSize::new(meta.size))
        .mtime(meta.mtime.map(ModificationTime::new))
        .ext(FileExtension::new(meta.ext.clone()))
        .name(FileName::new(meta.name.clone()));

    Some(builder.build())
}
