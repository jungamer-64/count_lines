// src/domain/compare.rs
mod snapshot;

use std::{collections::HashMap, path::Path};

use snapshot::{FileItem, Snapshot};

use crate::{
    error::{InfrastructureError, Result},
    infrastructure::persistence::FileReader,
};

/// Compare two JSON snapshot files and return a formatted diff. The
/// snapshots must be compatible with the output of `count_lines --format json`.
pub fn run(old_path: &Path, new_path: &Path) -> Result<String> {
    let old_file = FileReader::open(old_path).map_err(|source| InfrastructureError::FileRead {
        path: old_path.to_path_buf(),
        source,
    })?;
    let new_file = FileReader::open(new_path).map_err(|source| InfrastructureError::FileRead {
        path: new_path.to_path_buf(),
        source,
    })?;
    let old: Snapshot = serde_json::from_reader(old_file).map_err(InfrastructureError::from)?;
    let new: Snapshot = serde_json::from_reader(new_file).map_err(InfrastructureError::from)?;
    let comparison = SnapshotComparison::new(old, new);
    Ok(comparison.format())
}

/// A helper type that holds two snapshots and provides methods to
/// generate a human-readable comparison.
struct SnapshotComparison {
    old: Snapshot,
    new: Snapshot,
}

impl SnapshotComparison {
    fn new(old: Snapshot, new: Snapshot) -> Self {
        Self { old, new }
    }

    /// Render the full comparison as a string. The output mimics the
    /// style of the original tool, showing summary deltas and per-file
    /// differences.
    fn format(&self) -> String {
        let mut output = String::new();
        output.push_str("DIFF (new - old)\n");
        output.push_str(&self.format_summary_diff());
        output.push_str("\n[Changed files]\n");
        output.push_str(&self.format_file_diffs());
        output
    }

    /// Produce a formatted summary difference section. It reports
    /// changes in total lines, characters, files, and words (if
    /// available) between the two snapshots.
    fn format_summary_diff(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "Lines: {} -> {} (Δ {})\n",
            self.old.summary.lines,
            self.new.summary.lines,
            self.calculate_diff(self.new.summary.lines, self.old.summary.lines)
        ));
        output.push_str(&format!(
            "Chars: {} -> {} (Δ {})\n",
            self.old.summary.chars,
            self.new.summary.chars,
            self.calculate_diff(self.new.summary.chars, self.old.summary.chars)
        ));
        output.push_str(&format!(
            "Files: {} -> {} (Δ {})\n",
            self.old.summary.files,
            self.new.summary.files,
            self.calculate_diff(self.new.summary.files, self.old.summary.files)
        ));
        if let (Some(ow), Some(nw)) = (self.old.summary.words, self.new.summary.words) {
            output.push_str(&format!(
                "Words: {} -> {} (Δ {})\n",
                ow,
                nw,
                self.calculate_diff(nw, ow)
            ));
        }
        output
    }

    /// Format per-file differences. For each file present in the new
    /// snapshot, this reports the delta relative to the old snapshot.
    /// Added files are indicated explicitly.
    fn format_file_diffs(&self) -> String {
        // Build a lookup table from file name to its entry in the old snapshot
        let old_map: HashMap<&str, &FileItem> = self
            .old
            .files
            .iter()
            .map(|f| (f.file.as_str(), f))
            .collect();
        let mut output = String::new();
        for new_file in &self.new.files {
            if let Some(old_file) = old_map.get(new_file.file.as_str()) {
                let lines_diff = self.calculate_diff(new_file.lines, old_file.lines);
                let chars_diff = self.calculate_diff(new_file.chars, old_file.chars);
                let words_diff = match (old_file.words, new_file.words) {
                    (Some(ow), Some(nw)) => Some(self.calculate_diff(nw, ow)),
                    _ => None,
                };
                if lines_diff != 0 || chars_diff != 0 || words_diff.unwrap_or(0) != 0 {
                    if let Some(wd) = words_diff {
                        output.push_str(&format!(
                            "{:>10} L  {:>10} C  {:>10} W  {}\n",
                            lines_diff, chars_diff, wd, new_file.file
                        ));
                    } else {
                        output.push_str(&format!(
                            "{:>10} L  {:>10} C  {}\n",
                            lines_diff, chars_diff, new_file.file
                        ));
                    }
                }
            } else {
                // New file, show absolute values with added marker
                output.push_str(&format!(
                    "{:>10} L  {:>10} C  {} (added)\n",
                    new_file.lines as isize, new_file.chars as isize, new_file.file
                ));
            }
        }
        output
    }

    /// Compute a signed difference between two unsigned values.
    fn calculate_diff(&self, new: usize, old: usize) -> isize {
        new as isize - old as isize
    }
}
