use std::path::Path;

use crate::{
    application::commands::{
        AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
        SnapshotComparator,
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
    fn measure(&self, entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
        crate::infrastructure::measurement::measure_entries(entries, config)
    }
}

pub struct OutputEmitter;

impl FileStatisticsPresenter for OutputEmitter {
    fn present(&self, stats: &[FileStats], config: &Config) -> Result<()> {
        crate::infrastructure::io::output::emit(stats, config)
            .map_err(|err| InfrastructureError::OutputError(err.to_string()).into())
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
