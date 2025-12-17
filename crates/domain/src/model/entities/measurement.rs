use crate::model::FileStats;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct MeasurementOutcome {
    pub stats: Vec<FileStats>,
    pub changed_files: Vec<PathBuf>,
    pub removed_files: Vec<PathBuf>,
}

impl MeasurementOutcome {
    pub fn new(
        stats: Vec<FileStats>,
        changed_files: Vec<PathBuf>,
        removed_files: Vec<PathBuf>,
    ) -> Self {
        Self {
            stats,
            changed_files,
            removed_files,
        }
    }
}
