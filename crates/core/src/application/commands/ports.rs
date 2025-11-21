use std::path::Path;

use crate::{
    domain::{
        config::Config,
        model::{FileEntry, FileStats, MeasurementOutcome},
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
