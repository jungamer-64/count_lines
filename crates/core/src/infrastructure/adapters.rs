use std::path::Path;

use crate::{
    application::commands::{
        AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
        MeasurementOutcome, SnapshotComparator,
    },
    domain::{
        config::Config,
        model::{FileEntry, FileStats},
    },
    error::{InfrastructureError, Result},
};

pub struct FileSystemEntryProvider;

impl FileEntryProvider for FileSystemEntryProvider {
    fn collect(&self, config: &Config) -> Result<Vec<FileEntry>> {
        crate::infrastructure::filesystem::collect_entries(config)
    }
}

pub struct ParallelFileStatisticsProcessor;

impl FileStatisticsProcessor for ParallelFileStatisticsProcessor {
    fn measure(&self, entries: Vec<FileEntry>, config: &Config) -> Result<MeasurementOutcome> {
        crate::infrastructure::measurement::measure_entries(entries, config)
    }
}

pub struct OutputEmitter;

impl FileStatisticsPresenter for OutputEmitter {
    fn present(&self, stats: &[FileStats], config: &Config) -> Result<()> {
        crate::infrastructure::io::output::emit(stats, config)
            .map_err(|err| InfrastructureError::OutputError { message: err.to_string(), source: Some(Box::new(err)) }.into())
    }
}

pub struct JsonlWatchEmitter;

impl Default for JsonlWatchEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonlWatchEmitter {
    pub fn new() -> Self {
        Self
    }
}

impl FileStatisticsPresenter for JsonlWatchEmitter {
    fn present(&self, _stats: &[FileStats], _config: &Config) -> Result<()> {
        Ok(())
    }
}

pub struct SnapshotDiffAdapter;

impl SnapshotComparator for SnapshotDiffAdapter {
    fn compare(&self, old: &Path, new: &Path) -> Result<String> {
        crate::infrastructure::comparison::run(old, new)
    }
}

pub struct ConsoleNotifier;

impl AnalysisNotifier for ConsoleNotifier {
    fn info(&self, message: &str) {
        eprintln!("{message}");
    }

    fn warn(&self, message: &str) {
        eprintln!("{message}");
    }
}
