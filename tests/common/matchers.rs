//! アサーションヘルパー

use count_lines_core::domain::model::{FileStats, FileStatsV2};

/// FileStats用のカスタムアサーション
pub struct FileStatsAssertion {
    stats: FileStatsV2,
}

impl FileStatsAssertion {
    pub fn new(stats: &FileStats) -> Self {
        Self { stats: stats.to_v2() }
    }

    pub fn has_lines(self, expected: usize) -> Self {
        assert_eq!(
            self.stats.lines().value(),
            expected,
            "Expected {} lines, got {}",
            expected,
            self.stats.lines().value()
        );
        self
    }

    pub fn has_chars(self, expected: usize) -> Self {
        assert_eq!(
            self.stats.chars().value(),
            expected,
            "Expected {} chars, got {}",
            expected,
            self.stats.chars().value()
        );
        self
    }

    pub fn has_words(self, expected: Option<usize>) -> Self {
        let actual = self.stats.words().map(|w| w.value());
        assert_eq!(actual, expected, "Expected {:?} words, got {:?}", expected, actual);
        self
    }

    pub fn has_ext(self, expected: &str) -> Self {
        assert_eq!(
            self.stats.ext().as_str(),
            expected,
            "Expected ext '{}', got '{}'",
            expected,
            self.stats.ext().as_str()
        );
        self
    }
}

/// アサーションヘルパー関数
pub fn assert_stats(stats: &FileStats) -> FileStatsAssertion {
    FileStatsAssertion::new(stats)
}

/// ソート順の検証
pub fn assert_sorted_by_lines_desc(stats: &[FileStats]) {
    for window in stats.windows(2) {
        assert!(window[0].lines >= window[1].lines, "Stats not sorted by lines descending");
    }
}

pub fn assert_sorted_by_lines_asc(stats: &[FileStats]) {
    for window in stats.windows(2) {
        assert!(window[0].lines <= window[1].lines, "Stats not sorted by lines ascending");
    }
}

#[cfg(test)]
mod tests {
    use super::{super::builders::FileStatsBuilder, *};

    #[test]
    fn file_stats_builder_assertions_work() {
        let stats = FileStatsBuilder::new("test.rs").lines(10).chars(100).words(20).build();

        assert_stats(&stats).has_lines(10).has_chars(100).has_words(Some(20)).has_ext("rs");
    }
}
