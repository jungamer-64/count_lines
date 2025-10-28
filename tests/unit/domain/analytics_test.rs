use chrono::{Local, TimeZone};
use count_lines_core::domain::{
    analytics::Aggregator,
    config::ByKey,
    grouping::Granularity,
    model::{FileMeta, FileStats},
};
use std::path::Path;

fn make_stats(
    path: &str,
    lines: usize,
    chars: usize,
    ext: &str,
    mtime: Option<chrono::DateTime<Local>>,
) -> FileStats {
    let meta =
        FileMeta { size: 0, mtime, is_text: true, ext: ext.to_string(), name: Path::new(path).file_name().unwrap().to_string_lossy().into() };
    FileStats::new(path.into(), lines, chars, Some(lines / 2), &meta)
}

#[test]
fn aggregates_by_extension_and_sorts_descending() {
    let stats = vec![
        make_stats("src/lib.rs", 20, 120, "rs", None),
        make_stats("src/main.rs", 30, 180, "rs", None),
        make_stats("README.md", 10, 80, "md", None),
        make_stats("LICENSE", 5, 40, "", None),
    ];

    let aggregated = Aggregator::aggregate(&stats, &[ByKey::Ext]);
    assert_eq!(aggregated.len(), 1);
    let (label, groups) = &aggregated[0];
    assert_eq!(label, "By Extension");
    assert_eq!(groups.len(), 3);

    assert_eq!(groups[0].key, "rs");
    assert_eq!(groups[0].lines, 50);
    assert_eq!(groups[0].chars, 300);
    assert_eq!(groups[0].count, 2);

    assert_eq!(groups[1].key, "md");
    assert_eq!(groups[1].lines, 10);

    assert_eq!(groups[2].key, "(noext)");
    assert_eq!(groups[2].lines, 5);
}

#[test]
fn aggregates_by_directory_with_depth() {
    let stats = vec![
        make_stats("src/lib.rs", 12, 90, "rs", None),
        make_stats("src/bin/main.rs", 8, 60, "rs", None),
        make_stats("tests/unit.rs", 20, 150, "rs", None),
        make_stats("Cargo.toml", 3, 20, "toml", None),
    ];

    let aggregated = Aggregator::aggregate(&stats, &[ByKey::Dir(1)]);
    assert_eq!(aggregated.len(), 1);
    let (label, groups) = &aggregated[0];
    assert_eq!(label, "By Directory (depth=1)");

    assert_eq!(groups[0].key, "tests");
    assert_eq!(groups[0].lines, 20);
    assert_eq!(groups[0].count, 1);

    assert_eq!(groups[1].key, "src");
    assert_eq!(groups[1].lines, 20);
    assert_eq!(groups[1].count, 2);

    assert_eq!(groups[2].key, ".");
    assert_eq!(groups[2].lines, 3);
}

#[test]
fn aggregates_by_mtime_bucket() {
    let jan = Local.with_ymd_and_hms(2024, 1, 15, 12, 0, 0).unwrap();
    let feb = Local.with_ymd_and_hms(2024, 2, 5, 9, 0, 0).unwrap();

    let stats = vec![
        make_stats("src/lib.rs", 10, 80, "rs", Some(jan)),
        make_stats("src/main.rs", 5, 60, "rs", Some(jan)),
        make_stats("docs/guide.md", 7, 70, "md", Some(feb)),
        make_stats("README.md", 2, 20, "md", None),
    ];

    let aggregated = Aggregator::aggregate(&stats, &[ByKey::Mtime(Granularity::Month)]);
    let (label, groups) = &aggregated[0];
    assert_eq!(label, "By Mtime (month)");

    assert_eq!(groups[0].key, "2024-01");
    assert_eq!(groups[0].lines, 15);
    assert_eq!(groups[0].count, 2);

    assert_eq!(groups[1].key, "2024-02");
    assert_eq!(groups[1].lines, 7);

    assert_eq!(groups[2].key, "(no mtime)");
    assert_eq!(groups[2].lines, 2);
}
