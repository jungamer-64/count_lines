use super::FileStats;

/// Summary statistics over all processed files.
#[derive(Debug, Clone)]
pub struct Summary {
    pub lines: usize,
    pub chars: usize,
    pub words: usize,
    pub files: usize,
}

impl Summary {
    pub fn from_stats(stats: &[FileStats]) -> Self {
        let (lines, chars, words) = stats.iter().fold((0, 0, 0), |(l, c, w), s| {
            (l + s.lines, c + s.chars, w + s.words.unwrap_or(0))
        });
        Self {
            lines,
            chars,
            words,
            files: stats.len(),
        }
    }
}
