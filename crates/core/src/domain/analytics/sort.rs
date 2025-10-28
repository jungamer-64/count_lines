use crate::domain::{config::Config, model::FileStats, options::SortKey};
use std::cmp::Ordering;

/// 統計データをソートする
pub fn apply_sort(stats: &mut [FileStats], config: &Config) {
    // ソート不要なケースは早期リターン
    if config.total_only || config.summary_only || config.sort_specs.is_empty() {
        return;
    }

    // 安定ソートのため逆順に適用
    for (key, descending) in config.sort_specs.iter().rev() {
        stats.sort_by(|a, b| {
            let ordering = compare_stats(a, b, *key);
            if *descending {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }
}

/// 2つの統計データを比較
fn compare_stats(a: &FileStats, b: &FileStats, key: SortKey) -> Ordering {
    match key {
        SortKey::Lines => a.lines.cmp(&b.lines),
        SortKey::Chars => a.chars.cmp(&b.chars),
        SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
        SortKey::Name => a.path.cmp(&b.path),
        SortKey::Ext => a.ext.cmp(&b.ext),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        config::{ByKey, Config, Filters},
        model::FileMeta,
        options::OutputFormat,
    };
    use std::path::PathBuf;

    fn make_test_stats(name: &str, lines: usize) -> FileStats {
        let meta = FileMeta {
            size: 0,
            mtime: None,
            is_text: true,
            ext: String::new(),
            name: name.to_string(),
        };
        FileStats::new(PathBuf::from(name), lines, 0, None, &meta)
    }

    #[test]
    fn sorts_by_lines_descending() {
        let mut stats = vec![
            make_test_stats("a", 10),
            make_test_stats("b", 30),
            make_test_stats("c", 20),
        ];

        let config = Config {
            sort_specs: vec![(SortKey::Lines, true)],
            // ... その他のフィールド
            format: OutputFormat::Table,
            top_n: None,
            by_modes: vec![],
            summary_only: false,
            total_only: false,
            by_limit: None,
            filters: Filters::default(),
            hidden: false,
            follow: false,
            use_git: false,
            jobs: 1,
            no_default_prune: false,
            abs_path: false,
            abs_canonical: false,
            trim_root: None,
            words: false,
            count_newlines_in_chars: false,
            text_only: false,
            fast_text_detect: false,
            files_from: None,
            files_from0: None,
            paths: vec![],
            mtime_since: None,
            mtime_until: None,
            total_row: false,
            progress: false,
            ratio: false,
            output: None,
            strict: false,
            compare: None,
        };

        apply_sort(&mut stats, &config);

        assert_eq!(stats[0].lines, 30);
        assert_eq!(stats[1].lines, 20);
        assert_eq!(stats[2].lines, 10);
    }
}