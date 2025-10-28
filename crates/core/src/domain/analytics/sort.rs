// crates/core/src/domain/analytics/sort.rs
use std::cmp::Ordering;
use crate::domain::{
    model::FileStats,
    options::SortKey,
    config::Config, // 互換ラッパー用（不要なら外してOK）
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl From<bool> for SortOrder {
    #[inline]
    fn from(desc: bool) -> Self {
        if desc { SortOrder::Descending } else { SortOrder::Ascending }
    }
}

impl SortOrder {
    #[inline]
    pub fn apply(self, ordering: Ordering) -> Ordering {
        match self {
            SortOrder::Ascending => ordering,
            SortOrder::Descending => ordering.reverse(),
        }
    }
}

impl SortKey {
    #[inline]
    fn compare(&self, a: &FileStats, b: &FileStats) -> Ordering {
        match self {
            SortKey::Lines => a.lines.cmp(&b.lines),
            SortKey::Chars => a.chars.cmp(&b.chars),
            SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
            // 必要なら: 名前/拡張子を正規化してから比較
            SortKey::Name => a.path.cmp(&b.path),
            SortKey::Ext  => a.ext.cmp(&b.ext),
        }
    }
}

/// 複数キーの合成コンパレータで **1 回** の安定ソートに集約
pub fn apply_sort(stats: &mut [FileStats], specs: &[(SortKey, SortOrder)]) {
    if stats.is_empty() || specs.is_empty() {
        return;
    }
    stats.sort_by(|a, b| {
        for (key, order) in specs {
            let ord = key.compare(a, b);
            if ord != Ordering::Equal {
                return order.apply(ord);
            }
        }
        Ordering::Equal
    });
}

/// 互換レイヤ: 旧来の Config を受け取って適用
pub fn apply_sort_with_config(stats: &mut [FileStats], config: &Config) {
    // 旧仕様互換: total_only / summary_only のときはソートしない
    if config.total_only || config.summary_only || config.sort_specs.is_empty() {
        return;
    }
    // 旧: Vec<(SortKey, bool /*desc*/)> を新: Vec<(SortKey, SortOrder)> に変換
    let specs: Vec<(SortKey, SortOrder)> =
        config.sort_specs.iter().map(|(k, desc)| (*k, (*desc).into())).collect();
    apply_sort(stats, &specs);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // 最低限のテスト用コンストラクタ
    fn make(name: &str, lines: usize, ext: &str) -> FileStats {
        use crate::domain::model::FileMeta;
        let meta = FileMeta {
            size: 0,
            mtime: None,
            is_text: true,
            ext: ext.to_string(),
            name: name.to_string(),
        };
        FileStats::new(PathBuf::from(name), lines, 0, None, &meta)
    }

    #[test]
    fn sort_by_lines_desc() {
        let mut v = vec![make("a",10,"rs"), make("b",30,"rs"), make("c",20,"rs")];
        apply_sort(&mut v, &[(SortKey::Lines, SortOrder::Descending)]);
        assert_eq!(v.iter().map(|s| s.lines).collect::<Vec<_>>(), vec![30,20,10]);
    }

    #[test]
    fn sort_by_lines_then_name() {
        let mut v = vec![
            make("b", 10, "rs"),
            make("a", 10, "rs"),
            make("c", 20, "rs"),
        ];
        apply_sort(&mut v, &[
            (SortKey::Lines, SortOrder::Ascending),
            (SortKey::Name,  SortOrder::Ascending),
        ]);
        assert_eq!(v.iter().map(|s| (&s.lines, s.path.file_name().unwrap().to_str().unwrap()))
                    .collect::<Vec<_>>(),
                   vec![(&10, "a"), (&10, "b"), (&20, "c")]);
    }

    // プロパティテスト：同じ lines の群で相対順が保たれる（安定）
    // FileStats の Arbitrary 実装不要な戦略：lines を生成して name に連番を埋める
    mod prop {
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn stable_for_equal_lines(lines in proptest::collection::vec(0usize..1000, 0..200)) {
                // name に元のインデックスを埋め込んで識別子にする
                let mut v: Vec<FileStats> = lines.iter()
                    .enumerate()
                    .map(|(i, &ln)| make(&format!("id_{:04}", i), ln, "rs"))
                    .collect();

                apply_sort(&mut v, &[(SortKey::Lines, SortOrder::Ascending)]);

                // 同値群ごとに「名前の連番が昇順（= 元順）」であることを検証
                let mut i = 0;
                while i < v.len() {
                    let ln = v[i].lines;
                    let mut j = i + 1;
                    while j < v.len() && v[j].lines == ln { j += 1; }
                    // グループ [i, j) は全て同じ lines
                    let names: Vec<usize> = v[i..j].iter()
                        .map(|s| {
                            let n = s.path.file_name().unwrap().to_str().unwrap();
                            n.trim_start_matches("id_").parse::<usize>().unwrap()
                        })
                        .collect();
                    let mut sorted = names.clone();
                    sorted.sort_unstable(); // 元順は連番昇順
                    prop_assert_eq!(names, sorted);
                    i = j;
                }
            }
        }
    }
}
