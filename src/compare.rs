use crate::error::{AppError, Result};
use crate::stats::FileStats;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

/// Safely convert usize to isize, capping at isize::MAX to avoid wrap-around
fn to_isize(value: usize) -> isize {
    isize::try_from(value).unwrap_or(isize::MAX)
}

/// Safely calculate the difference between two usize values as isize
fn safe_diff(new_val: usize, old_val: usize) -> isize {
    if new_val >= old_val {
        to_isize(new_val - old_val)
    } else {
        -to_isize(old_val - new_val)
    }
}

pub struct ComparisonSummary {
    pub added_files: usize,
    pub removed_files: usize,
    pub modified_files: usize,
    pub unchanged_files: usize,
    pub diff_lines: isize,
    pub diff_chars: isize,
    pub diff_words: isize,
}

pub enum FileDiff<'a> {
    Added(&'a FileStats),
    Removed(&'a FileStats),
    Modified {
        path: &'a PathBuf,
        old_lines: usize,
        new_lines: usize,
        old_chars: usize,
        new_chars: usize,
    },
}

pub fn compare_snapshots(old_path: &PathBuf, new_path: &PathBuf) -> Result<()> {
    let old_stats = load_stats(old_path)?;
    let new_stats = load_stats(new_path)?;

    let (diffs, summary) = compare_stats(&old_stats, &new_stats);

    print_comparison_results(&diffs, &summary, &old_stats, &new_stats);

    Ok(())
}

fn compare_stats<'a>(
    old_stats: &'a [FileStats],
    new_stats: &'a [FileStats],
) -> (Vec<FileDiff<'a>>, ComparisonSummary) {
    let old_map: HashMap<PathBuf, &FileStats> =
        old_stats.iter().map(|s| (s.path.clone(), s)).collect();
    let new_map: HashMap<PathBuf, &FileStats> =
        new_stats.iter().map(|s| (s.path.clone(), s)).collect();

    let mut diffs = Vec::new();
    let mut summary = ComparisonSummary {
        added_files: 0,
        removed_files: 0,
        modified_files: 0,
        unchanged_files: 0,
        diff_lines: 0,
        diff_chars: 0,
        diff_words: 0,
    };

    // Check old entries (Modified and Removed)
    for (path, old_s) in &old_map {
        if let Some(new_s) = new_map.get(path) {
            // Compare
            if old_s.lines != new_s.lines
                || old_s.chars != new_s.chars
                || old_s.words != new_s.words
            {
                diffs.push(FileDiff::Modified {
                    path: &old_s.path,
                    old_lines: old_s.lines,
                    new_lines: new_s.lines,
                    old_chars: old_s.chars,
                    new_chars: new_s.chars,
                });
                summary.modified_files += 1;
                summary.diff_lines += safe_diff(new_s.lines, old_s.lines);
                summary.diff_chars += safe_diff(new_s.chars, old_s.chars);
                if let (Some(w1), Some(w2)) = (old_s.words, new_s.words) {
                    summary.diff_words += safe_diff(w2, w1);
                }
            } else {
                summary.unchanged_files += 1;
            }
        } else {
            diffs.push(FileDiff::Removed(old_s));
            summary.removed_files += 1;
            summary.diff_lines -= to_isize(old_s.lines);
            summary.diff_chars -= to_isize(old_s.chars);
            if let Some(w) = old_s.words {
                summary.diff_words -= to_isize(w);
            }
        }
    }

    // Check new entries (Added)
    for (path, new_s) in &new_map {
        if !old_map.contains_key(path) {
            diffs.push(FileDiff::Added(new_s));
            summary.added_files += 1;
            summary.diff_lines += to_isize(new_s.lines);
            summary.diff_chars += to_isize(new_s.chars);
            if let Some(w) = new_s.words {
                summary.diff_words += to_isize(w);
            }
        }
    }

    // Sort by path for consistent output
    diffs.sort_by(|a, b| {
        let p1 = match a {
            FileDiff::Added(s) => &s.path,
            FileDiff::Removed(s) => &s.path,
            FileDiff::Modified { path, .. } => path,
        };
        let p2 = match b {
            FileDiff::Added(s) => &s.path,
            FileDiff::Removed(s) => &s.path,
            FileDiff::Modified { path, .. } => path,
        };
        p1.cmp(p2)
    });

    (diffs, summary)
}

fn print_comparison_results(
    diffs: &[FileDiff],
    summary: &ComparisonSummary,
    old_stats: &[FileStats],
    new_stats: &[FileStats],
) {
    // Print Summary
    println!("Comparison Summary");
    println!("-------------------");
    println!(
        "Files: +{} -{} ~{} ({} unchanged)",
        summary.added_files, summary.removed_files, summary.modified_files, summary.unchanged_files
    );
    println!("Lines: {:+}", summary.diff_lines);
    println!("Chars: {:+}", summary.diff_chars);

    let show_words =
        old_stats.iter().any(|s| s.words.is_some()) && new_stats.iter().any(|s| s.words.is_some());
    if show_words {
        println!("Words: {:+}", summary.diff_words);
    }
    println!();

    let mut added_sections = Vec::new();
    let mut removed_sections = Vec::new();
    let mut modified_sections = Vec::new();

    for diff in diffs {
        match diff {
            FileDiff::Added(s) => added_sections.push(s),
            FileDiff::Removed(s) => removed_sections.push(s),
            FileDiff::Modified { .. } => modified_sections.push(diff),
        }
    }

    if !added_sections.is_empty() {
        println!("### Added Files");
        for s in added_sections {
            println!("+ {} (L:{}, C:{})", s.path.display(), s.lines, s.chars);
        }
        println!();
    }

    if !removed_sections.is_empty() {
        println!("### Removed Files");
        for s in removed_sections {
            println!("- {} (L:{}, C:{})", s.path.display(), s.lines, s.chars);
        }
        println!();
    }

    if !modified_sections.is_empty() {
        println!("### Modified Files");
        for diff in modified_sections {
            if let FileDiff::Modified {
                path,
                old_lines,
                new_lines,
                old_chars,
                new_chars,
            } = diff
            {
                let dl = safe_diff(*new_lines, *old_lines);
                let dc = safe_diff(*new_chars, *old_chars);
                println!("~ {} (Lines: {:+}, Chars: {:+})", path.display(), dl, dc);
            }
        }
    }
}

fn load_stats(path: &PathBuf) -> Result<Vec<FileStats>> {
    let file = File::open(path).map_err(AppError::Io)?;
    let reader = BufReader::new(file);
    let stats: Vec<FileStats> = serde_json::from_reader(reader)
        .map_err(|e| AppError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_identical() {
        let stats = vec![FileStats {
            lines: 10,
            path: PathBuf::from("a.rs"),
            ..Default::default()
        }];
        let (diffs, summary) = compare_stats(&stats, &stats);
        assert!(diffs.is_empty());
        assert_eq!(summary.added_files, 0);
        assert_eq!(summary.removed_files, 0);
        assert_eq!(summary.modified_files, 0);
        assert_eq!(summary.diff_lines, 0);
        assert_eq!(summary.unchanged_files, 1);
    }

    #[test]
    fn test_compare_added() {
        let old = vec![];
        let new = vec![FileStats {
            lines: 10,
            path: PathBuf::from("a.rs"),
            ..Default::default()
        }];
        let (diffs, summary) = compare_stats(&old, &new);
        assert_eq!(diffs.len(), 1);
        match &diffs[0] {
            FileDiff::Added(s) => assert_eq!(s.lines, 10),
            _ => panic!("Expected Added"),
        }
        assert_eq!(summary.added_files, 1);
        assert_eq!(summary.diff_lines, 10);
    }

    #[test]
    fn test_compare_removed() {
        let old = vec![FileStats {
            lines: 10,
            path: PathBuf::from("a.rs"),
            ..Default::default()
        }];
        let new = vec![];
        let (diffs, summary) = compare_stats(&old, &new);
        assert_eq!(diffs.len(), 1);
        match &diffs[0] {
            FileDiff::Removed(s) => assert_eq!(s.lines, 10),
            _ => panic!("Expected Removed"),
        }
        assert_eq!(summary.removed_files, 1);
        assert_eq!(summary.diff_lines, -10);
    }

    #[test]
    fn test_compare_modified() {
        let old = vec![FileStats {
            lines: 10,
            path: PathBuf::from("a.rs"),
            ..Default::default()
        }];
        let new = vec![FileStats {
            lines: 15,
            path: PathBuf::from("a.rs"),
            ..Default::default()
        }];
        let (diffs, summary) = compare_stats(&old, &new);
        assert_eq!(diffs.len(), 1);
        match &diffs[0] {
            FileDiff::Modified {
                path: _,
                old_lines,
                new_lines,
                ..
            } => {
                assert_eq!(*old_lines, 10);
                assert_eq!(*new_lines, 15);
            }
            _ => panic!("Expected Modified"),
        }
        assert_eq!(summary.modified_files, 1);
        assert_eq!(summary.diff_lines, 5);
    }
}
