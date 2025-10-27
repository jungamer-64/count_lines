use crate::domain::config::Config;
use crate::domain::model::FileStats;
use crate::domain::options::SortKey;
use std::cmp::Ordering;

/// Apply sorting to file statistics in-place based on configuration.
pub fn apply_sort(stats: &mut [FileStats], config: &Config) {
    if config.total_only || config.summary_only || config.sort_specs.is_empty() {
        return;
    }
    for (key, desc) in config.sort_specs.iter().rev() {
        stats.sort_by(|a, b| {
            let ord = Sorter::compare(a, b, *key);
            if *desc { ord.reverse() } else { ord }
        });
    }
}

struct Sorter;

impl Sorter {
    fn compare(a: &FileStats, b: &FileStats, key: SortKey) -> Ordering {
        match key {
            SortKey::Lines => a.lines.cmp(&b.lines),
            SortKey::Chars => a.chars.cmp(&b.chars),
            SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
            SortKey::Name => a.path.cmp(&b.path),
            SortKey::Ext => a.ext.cmp(&b.ext),
        }
    }
}
