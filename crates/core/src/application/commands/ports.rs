use std::path::{Path, PathBuf};

use crate::{
    domain::{
        config::Config,
        model::{FileEntry, FileStats},
    },
    error::Result,
};

pub trait SnapshotComparator {
    fn compare(&self, old: &Path, new: &Path) -> Result<String>;
}

pub trait FileEntryProvider {
    fn collect(&self, config: &Config) -> Result<Vec<FileEntry>>;
}

pub trait FileStatisticsProcessor {
    fn measure(&self, entries: Vec<FileEntry>, config: &Config) -> Result<MeasurementOutcome>;
}

pub trait FileStatisticsPresenter {
    fn present(&self, stats: &[FileStats], config: &Config) -> Result<()>;
}

pub trait AnalysisNotifier {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
}

#[derive(Debug, Clone)]
pub struct MeasurementOutcome {
    pub stats: Vec<FileStats>,
    pub changed_files: Vec<PathBuf>,
    pub removed_files: Vec<PathBuf>,
}

impl MeasurementOutcome {
    pub fn new(stats: Vec<FileStats>, changed_files: Vec<PathBuf>, removed_files: Vec<PathBuf>) -> Self {
        Self { stats, changed_files, removed_files }
    }
}
