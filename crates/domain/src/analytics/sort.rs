// domain analytics sorting utilities
use std::cmp::Ordering;

use crate::{
    model::{FileStats, FileStatsV2},
    options::SortKey,
};

/// ソート順序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    #[inline]
    pub fn apply(self, ordering: Ordering) -> Ordering {
        match self {
            Self::Ascending => ordering,
            Self::Descending => ordering.reverse(),
        }
    }
}

impl From<bool> for SortOrder {
    #[inline]
    fn from(desc: bool) -> Self {
        if desc {
            Self::Descending
        } else {
            Self::Ascending
        }
    }
}

/// ソート仕様を表す値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
    key: SortKey,
    order: SortOrder,
}

impl SortSpec {
    pub fn new(key: SortKey, order: SortOrder) -> Self {
        Self { key, order }
    }

    pub fn ascending(key: SortKey) -> Self {
        Self::new(key, SortOrder::Ascending)
    }

    pub fn descending(key: SortKey) -> Self {
        Self::new(key, SortOrder::Descending)
    }

    pub fn key(&self) -> SortKey {
        self.key
    }

    pub fn order(&self) -> SortOrder {
        self.order
    }
}

/// ソート戦略パターン実装
#[derive(Debug, Clone)]
pub struct SortStrategy {
    specs: Vec<SortSpec>,
}

impl SortStrategy {
    /// 新しいソート戦略を作成
    pub fn new(specs: Vec<SortSpec>) -> Self {
        Self { specs }
    }

    /// レガシーフォーマットから変換
    pub fn from_legacy(specs: Vec<(SortKey, bool)>) -> Self {
        let specs = specs
            .into_iter()
            .map(|(key, desc)| SortSpec::new(key, desc.into()))
            .collect();
        Self::new(specs)
    }

    /// デフォルト戦略（行数昇順）
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::new(vec![SortSpec::ascending(SortKey::Lines)])
    }

    /// ソート仕様が空かどうか
    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }

    /// ファイル統計をソート（インプレース）
    pub fn apply(&self, stats: &mut [FileStats]) {
        if stats.is_empty() || self.specs.is_empty() {
            return;
        }

        let mut converted: Vec<_> = stats.iter().map(|s| s.to_v2()).collect();
        converted.sort_by(|a, b| self.compare(a, b));

        for (legacy, v2) in stats.iter_mut().zip(converted.into_iter()) {
            *legacy = v2.to_legacy();
        }
    }

    /// ソートされた新しいベクタを返す
    pub fn sorted(&self, stats: Vec<FileStats>) -> Vec<FileStats> {
        if stats.is_empty() || self.specs.is_empty() {
            return stats;
        }

        let mut converted: Vec<_> = stats.into_iter().map(|s| s.to_v2()).collect();
        converted.sort_by(|a, b| self.compare(a, b));
        converted.into_iter().map(|s| s.to_legacy()).collect()
    }

    /// 2つのFileStatsを比較
    fn compare(&self, a: &FileStatsV2, b: &FileStatsV2) -> Ordering {
        for spec in &self.specs {
            let cmp = spec.key.compare(a, b);
            if cmp != Ordering::Equal {
                return spec.order.apply(cmp);
            }
        }
        Ordering::Equal
    }
}

impl Default for SortStrategy {
    fn default() -> Self {
        Self::new(vec![SortSpec::ascending(SortKey::Lines)])
    }
}

impl SortKey {
    /// FileStatsを比較
    #[inline]
    pub fn compare(&self, a: &FileStatsV2, b: &FileStatsV2) -> Ordering {
        match self {
            Self::Lines => a.lines().cmp(&b.lines()),
            Self::Chars => a.chars().cmp(&b.chars()),
            Self::Words => {
                let a_words = a.words().unwrap_or_default();
                let b_words = b.words().unwrap_or_default();
                a_words.cmp(&b_words)
            }
            Self::Sloc => {
                let a_sloc = a.sloc().unwrap_or_default();
                let b_sloc = b.sloc().unwrap_or_default();
                a_sloc.cmp(&b_sloc)
            }
            Self::Size => a.size().cmp(&b.size()),
            Self::Name => a.path().as_path().cmp(b.path().as_path()),
            Self::Ext => a.ext().cmp(b.ext()),
        }
    }
}

// ============================================================================
// 後方互換性レイヤー
// ============================================================================

/// 互換性のための関数（既存コードの段階的移行用）
pub fn apply_sort(stats: &mut [FileStats], specs: &[(SortKey, SortOrder)]) {
    let strategy = SortStrategy::new(specs.iter().map(|(k, o)| SortSpec::new(*k, *o)).collect());
    strategy.apply(stats);
}

/// Config を使った互換レイヤー
pub fn apply_sort_with_config(stats: &mut [FileStats], config: &crate::config::Config) {
    if config.total_only || config.summary_only || config.sort_specs.is_empty() {
        return;
    }

    let strategy = SortStrategy::from_legacy(config.sort_specs.clone());
    strategy.apply(stats);
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::value_objects::{CharCount, FilePath, FileSize, LineCount};

    fn make_stats(name: &str, lines: usize, chars: usize) -> FileStats {
        FileStatsV2::builder(FilePath::new(PathBuf::from(name)))
            .lines(LineCount::new(lines))
            .chars(CharCount::new(chars))
            .words(None)
            .build()
            .to_legacy()
    }

    #[test]
    fn sort_by_lines_descending() {
        let mut stats = vec![
            make_stats("a.txt", 10, 100),
            make_stats("b.txt", 30, 300),
            make_stats("c.txt", 20, 200),
        ];

        let strategy = SortStrategy::new(vec![SortSpec::descending(SortKey::Lines)]);
        strategy.apply(&mut stats);

        let lines: Vec<_> = stats.iter().map(|s| s.lines).collect();
        assert_eq!(lines, vec![30, 20, 10]);
    }

    #[test]
    fn sort_by_multiple_keys() {
        let mut stats = vec![
            make_stats("b.txt", 10, 200),
            make_stats("a.txt", 10, 100),
            make_stats("c.txt", 20, 300),
        ];

        let strategy = SortStrategy::new(vec![
            SortSpec::ascending(SortKey::Lines),
            SortSpec::ascending(SortKey::Chars),
        ]);
        strategy.apply(&mut stats);

        let names: Vec<_> = stats
            .iter()
            .map(|s| s.path.to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn sort_by_size_descending() {
        fn make_stats_with_size(name: &str, size: u64) -> FileStats {
            FileStatsV2::builder(FilePath::new(PathBuf::from(name)))
                .lines(LineCount::new(1))
                .chars(CharCount::new(1))
                .size(FileSize::new(size))
                .build()
                .to_legacy()
        }

        let mut stats = vec![
            make_stats_with_size("small.txt", 10),
            make_stats_with_size("large.txt", 100),
        ];
        let strategy = SortStrategy::new(vec![SortSpec::descending(SortKey::Size)]);

        strategy.apply(&mut stats);

        let names: Vec<_> = stats
            .iter()
            .map(|s| s.path.to_string_lossy().to_string())
            .collect();
        assert_eq!(names, vec!["large.txt", "small.txt"]);
    }

    #[test]
    fn stable_sort_preserves_order() {
        let mut stats = vec![
            make_stats("file1.txt", 10, 100),
            make_stats("file2.txt", 10, 100),
            make_stats("file3.txt", 10, 100),
        ];

        let original_names: Vec<_> = stats
            .iter()
            .map(|s| s.path.to_string_lossy().to_string())
            .collect();

        let strategy = SortStrategy::new(vec![SortSpec::ascending(SortKey::Lines)]);
        strategy.apply(&mut stats);

        let sorted_names: Vec<_> = stats
            .iter()
            .map(|s| s.path.to_string_lossy().to_string())
            .collect();

        assert_eq!(original_names, sorted_names);
    }

    #[test]
    fn empty_strategy_does_nothing() {
        let mut stats = vec![
            make_stats("c.txt", 30, 300),
            make_stats("a.txt", 10, 100),
            make_stats("b.txt", 20, 200),
        ];

        let original_order = stats.clone();
        let strategy = SortStrategy::new(vec![]);
        strategy.apply(&mut stats);

        assert_eq!(stats.len(), original_order.len());
        for (a, b) in stats.iter().zip(original_order.iter()) {
            assert_eq!(a.lines, b.lines);
        }
    }

    #[test]
    fn sorted_returns_new_sorted_vector() {
        let original = vec![
            make_stats("b.txt", 5, 50),
            make_stats("a.txt", 10, 100),
            make_stats("c.txt", 1, 10),
        ];

        let strategy = SortStrategy::new(vec![SortSpec::ascending(SortKey::Lines)]);
        let sorted = strategy.sorted(original.clone());

        let sorted_lines: Vec<_> = sorted.iter().map(|s| s.lines).collect();
        assert_eq!(sorted_lines, vec![1, 5, 10]);

        let original_lines: Vec<_> = original.iter().map(|s| s.lines).collect();
        assert_eq!(
            original_lines,
            vec![5, 10, 1],
            "sorted should not mutate the input vector copy"
        );
    }

    #[test]
    fn default_strategy_matches_default_impl() {
        let via_assoc = SortStrategy::default();
        let via_trait: SortStrategy = Default::default();

        let mut stats_a = vec![
            make_stats("alpha.txt", 5, 50),
            make_stats("beta.txt", 10, 100),
        ];
        let mut stats_b = stats_a.clone();

        via_assoc.apply(&mut stats_a);
        via_trait.apply(&mut stats_b);

        assert_eq!(stats_a, stats_b);
    }

    #[test]
    fn sort_order_conversion() {
        assert_eq!(SortOrder::from(true), SortOrder::Descending);
        assert_eq!(SortOrder::from(false), SortOrder::Ascending);
    }

    #[test]
    fn sort_order_apply() {
        let asc = SortOrder::Ascending;
        let desc = SortOrder::Descending;

        assert_eq!(asc.apply(Ordering::Less), Ordering::Less);
        assert_eq!(desc.apply(Ordering::Less), Ordering::Greater);
    }

    // プロパティベーステスト用のヘルパー
    #[cfg(test)]
    mod property_tests {
        use super::*;

        #[test]
        fn sorted_stays_sorted() {
            // 任意のFileStatsリストをソートして、
            // 結果が実際にソートされているか確認
            let mut stats = vec![
                make_stats("z.txt", 100, 1000),
                make_stats("a.txt", 1, 10),
                make_stats("m.txt", 50, 500),
            ];

            let strategy = SortStrategy::new(vec![SortSpec::ascending(SortKey::Lines)]);
            strategy.apply(&mut stats);

            // ソート済みかチェック
            for window in stats.windows(2) {
                assert!(window[0].lines <= window[1].lines);
            }
        }

        #[test]
        fn sort_is_deterministic() {
            let original = vec![
                make_stats("file1.txt", 30, 300),
                make_stats("file2.txt", 10, 100),
                make_stats("file3.txt", 20, 200),
            ];

            let strategy = SortStrategy::new(vec![SortSpec::ascending(SortKey::Lines)]);

            let mut first_run = original.clone();
            strategy.apply(&mut first_run);

            let mut second_run = original.clone();
            strategy.apply(&mut second_run);

            // 2回ソートしても同じ結果
            for (a, b) in first_run.iter().zip(second_run.iter()) {
                assert_eq!(a.lines, b.lines);
                assert_eq!(a.path, b.path);
            }
        }
    }
}
