use std::path::PathBuf;

use chrono::Local;
use count_lines_core::domain::model::{FileMeta, FileStats, Summary};

fn make_stats(path: &str, lines: usize, chars: usize, words: Option<usize>) -> FileStats {
    let meta = FileMeta {
        size: 128,
        mtime: Some(Local::now()),
        is_text: true,
        ext: "txt".into(),
        name: "file".into(),
    };
    FileStats::new(PathBuf::from(path), lines, chars, words, &meta)
}

#[test]
fn summary_aggregates_metrics() {
    let stats = vec![
        make_stats("a.txt", 10, 80, Some(5)),
        make_stats("b.txt", 5, 20, None),
        make_stats("c.txt", 0, 0, Some(2)),
    ];

    let summary = Summary::from_stats(&stats);
    assert_eq!(summary.lines, 15);
    assert_eq!(summary.chars, 100);
    assert_eq!(summary.words, 7); // missing words default to zero
    assert_eq!(summary.files, 3);
}
