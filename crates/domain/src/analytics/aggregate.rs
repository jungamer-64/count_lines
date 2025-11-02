use std::{
    collections::HashMap,
    path::{Component, Path},
};

use chrono::{DateTime, Local};

use crate::{config::ByKey, grouping::Granularity, model::FileStats};

/// Aggregation results for a single grouping key.
#[derive(Debug, Clone)]
pub struct AggregationGroup {
    pub key: String,
    pub lines: usize,
    pub chars: usize,
    pub count: usize,
}

impl AggregationGroup {
    fn new(key: String, lines: usize, chars: usize, count: usize) -> Self {
        Self { key, lines, chars, count }
    }
}

/// Aggregator to group and summarise file statistics by requested keys.
pub struct Aggregator;

impl Aggregator {
    pub fn aggregate(stats: &[FileStats], by_keys: &[ByKey]) -> Vec<(String, Vec<AggregationGroup>)> {
        by_keys.iter().map(|key| Self::aggregate_by_key(stats, key)).collect()
    }

    fn aggregate_by_key(stats: &[FileStats], key: &ByKey) -> (String, Vec<AggregationGroup>) {
        match key {
            ByKey::Ext => Self::aggregate_by_ext(stats),
            ByKey::Dir(depth) => Self::aggregate_by_dir(stats, *depth),
            ByKey::Mtime(gran) => Self::aggregate_by_mtime(stats, *gran),
        }
    }

    fn aggregate_by_ext(stats: &[FileStats]) -> (String, Vec<AggregationGroup>) {
        let map = Self::build_aggregation_map(stats, |s| {
            if s.ext.is_empty() { "(noext)".to_string() } else { s.ext.clone() }
        });
        ("By Extension".to_string(), Self::map_to_sorted_groups(map))
    }

    fn aggregate_by_dir(stats: &[FileStats], depth: usize) -> (String, Vec<AggregationGroup>) {
        let map = Self::build_aggregation_map(stats, |s| get_dir_key(&s.path, depth));
        (format!("By Directory (depth={depth})"), Self::map_to_sorted_groups(map))
    }

    fn aggregate_by_mtime(stats: &[FileStats], gran: Granularity) -> (String, Vec<AggregationGroup>) {
        let map = Self::build_aggregation_map(stats, |s| {
            s.mtime.map(|mt| mtime_bucket(mt, gran)).unwrap_or_else(|| "(no mtime)".to_string())
        });
        let gran_label = match gran {
            Granularity::Day => "day",
            Granularity::Week => "week",
            Granularity::Month => "month",
        };
        (format!("By Mtime ({gran_label})"), Self::map_to_sorted_groups(map))
    }

    fn build_aggregation_map<F>(stats: &[FileStats], key_fn: F) -> HashMap<String, (usize, usize, usize)>
    where
        F: Fn(&FileStats) -> String,
    {
        let mut map: HashMap<String, (usize, usize, usize)> = HashMap::new();
        for stat in stats {
            let key = key_fn(stat);
            let entry = map.entry(key).or_insert((0, 0, 0));
            entry.0 += stat.lines;
            entry.1 += stat.chars;
            entry.2 += 1;
        }
        map
    }

    fn map_to_sorted_groups(map: HashMap<String, (usize, usize, usize)>) -> Vec<AggregationGroup> {
        let mut groups: Vec<AggregationGroup> = map
            .into_iter()
            .map(|(key, (lines, chars, count))| AggregationGroup::new(key, lines, chars, count))
            .collect();
        // Sort primarily by lines (desc). For deterministic ordering when
        // line counts are equal, sort by key (desc) to keep results stable
        // across runs and platforms.
        groups.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| b.key.cmp(&a.key)));
        groups
    }
}

fn mtime_bucket(dt: DateTime<Local>, gran: Granularity) -> String {
    use chrono::Datelike;
    match gran {
        Granularity::Day => dt.format("%Y-%m-%d").to_string(),
        Granularity::Week => format!("{:04}-W{:02}", dt.iso_week().year(), dt.iso_week().week()),
        Granularity::Month => dt.format("%Y-%m").to_string(),
    }
}

fn get_dir_key(path: &Path, depth: usize) -> String {
    let base = path.parent().unwrap_or(Path::new("."));
    let parts: Vec<String> = base
        .components()
        .filter_map(|c| match c {
            Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .take(depth)
        .collect();
    if parts.is_empty() { ".".to_string() } else { parts.join("/") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::ByKey,
        grouping::Granularity,
        model::{FileStats, FileStatsBuilder},
        value_objects::{
            CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
        },
    };

    fn build_stat(
        path: impl Into<std::path::PathBuf>,
        ext: impl Into<String>,
        lines: usize,
        chars: usize,
        words: Option<usize>,
        mtime: Option<chrono::DateTime<chrono::Local>>,
    ) -> FileStats {
        FileStatsBuilder::new(FilePath::new(path.into()))
            .lines(LineCount::new(lines))
            .chars(CharCount::new(chars))
            .words(words.map(WordCount::new))
            .size(FileSize::new((chars * 2) as u64))
            .ext(FileExtension::new(ext.into()))
            .name(FileName::new("sample".to_string()))
            .mtime(mtime.map(ModificationTime::new))
            .build()
            .to_legacy()
    }

    #[test]
    fn aggregates_by_extension_sorted_by_lines_desc() {
        let stats = vec![
            build_stat("src/lib.rs", "rs", 50, 500, None, None),
            build_stat("src/main.rs", "rs", 100, 1_000, None, None),
            build_stat("README", "", 20, 200, None, None),
        ];

        let groups = Aggregator::aggregate(&stats, &[ByKey::Ext]);
        assert_eq!(groups.len(), 1);
        let (label, rows) = &groups[0];
        assert_eq!(label, "By Extension");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].key, "rs");
        assert_eq!(rows[0].lines, 150);
        assert_eq!(rows[0].chars, 1_500);
        assert_eq!(rows[0].count, 2);
        assert_eq!(rows[1].key, "(noext)");
        assert_eq!(rows[1].count, 1);
    }

    #[test]
    fn aggregates_by_directory_respect_depth() {
        let stats = vec![
            build_stat("src/lib.rs", "rs", 10, 100, None, None),
            build_stat("src/utils/helpers.rs", "rs", 20, 200, None, None),
            build_stat("tests/integration/main.rs", "rs", 30, 300, None, None),
        ];

        let groups = Aggregator::aggregate(&stats, &[ByKey::Dir(1)]);
        let (_, rows) = &groups[0];
        // Expect two top-level directories at depth 1: 'tests' and 'src'
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].key, "tests");
        assert_eq!(rows[0].lines, 30);
        assert_eq!(rows[1].key, "src");
        assert_eq!(rows[1].lines, 30);
    }

    #[test]
    fn aggregates_by_mtime_bucket_counts_files() {
        let now = chrono::Local::now();
        let yesterday = now - chrono::Duration::days(1);
        let stats = vec![
            build_stat("src/lib.rs", "rs", 10, 100, None, Some(now)),
            build_stat("src/main.rs", "rs", 20, 200, None, Some(yesterday)),
        ];

        let groups = Aggregator::aggregate(&stats, &[ByKey::Mtime(Granularity::Day)]);
        let (_, rows) = &groups[0];
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].count, 1);
        assert_eq!(rows[1].count, 1);
    }

    #[test]
    fn aggregate_returns_empty_without_keys() {
        let stats = vec![build_stat("src/lib.rs", "rs", 10, 100, None, None)];
        let groups = Aggregator::aggregate(&stats, &[]);
        assert!(groups.is_empty());
    }
}
