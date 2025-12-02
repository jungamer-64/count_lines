use crate::model::FileStats;

/// Summary statistics over all processed files.
#[derive(Debug, Clone)]
pub struct Summary {
    pub lines: usize,
    pub chars: usize,
    pub words: usize,
    pub sloc: usize,
    pub files: usize,
}

impl Summary {
    pub fn from_stats(stats: &[FileStats]) -> Self {
        let (lines, chars, words, sloc) = stats.iter().fold((0, 0, 0, 0), |(l, c, w, s), stat| {
            (l + stat.lines, c + stat.chars, w + stat.words.unwrap_or(0), s + stat.sloc.unwrap_or(0))
        });
        Self { lines, chars, words, sloc, files: stats.len() }
    }
}
