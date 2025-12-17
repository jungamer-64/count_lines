#![allow(dead_code)]
// tests/common/mocks.rs
//! テスト用モック実装

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use count_lines_core::{
    application::commands::{
        AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
        MeasurementOutcome, SnapshotComparator,
    },
    domain::{
        config::Config,
        model::{FileEntry, FileStats},
    },
    error::*,
};

// ============================================================================
// MockFileEntryProvider
// ============================================================================

pub struct MockFileEntryProvider {
    entries: Vec<FileEntry>,
    should_fail: bool,
}

impl MockFileEntryProvider {
    pub fn new(entries: Vec<FileEntry>) -> Self {
        Self {
            entries,
            should_fail: false,
        }
    }

    pub fn empty() -> Self {
        Self::new(vec![])
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl FileEntryProvider for MockFileEntryProvider {
    fn collect(&self, _config: &Config) -> Result<Vec<FileEntry>> {
        if self.should_fail {
            Err(InfrastructureError::FileSystemOperation {
                operation: "collect".to_string(),
                path: Path::new(".").to_path_buf(),
                source: std::io::Error::other("mock failure"),
            }
            .into())
        } else {
            Ok(self.entries.clone())
        }
    }
}

// ============================================================================
// MockFileStatisticsProcessor
// ============================================================================

pub enum ProcessorBehavior {
    Success(MeasurementOutcome),
    Failure(String),
}

pub struct MockFileStatisticsProcessor {
    behavior: ProcessorBehavior,
}

impl MockFileStatisticsProcessor {
    pub fn success(stats: Vec<FileStats>) -> Self {
        Self {
            behavior: ProcessorBehavior::Success(MeasurementOutcome::new(
                stats,
                Vec::new(),
                Vec::new(),
            )),
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            behavior: ProcessorBehavior::Failure(message.into()),
        }
    }

    pub fn empty() -> Self {
        Self::success(vec![])
    }
}

impl FileStatisticsProcessor for MockFileStatisticsProcessor {
    fn measure(&self, _entries: Vec<FileEntry>, _config: &Config) -> Result<MeasurementOutcome> {
        match &self.behavior {
            ProcessorBehavior::Success(outcome) => Ok(outcome.clone()),
            ProcessorBehavior::Failure(msg) => Err(InfrastructureError::MeasurementError {
                path: Path::new("mock").to_path_buf(),
                reason: msg.clone(),
            }
            .into()),
        }
    }
}

// ============================================================================
// RecordingPresenter
// ============================================================================

#[derive(Clone)]
pub struct RecordingPresenter {
    calls: Arc<Mutex<Vec<PresentCall>>>,
}

#[derive(Debug, Clone)]
pub struct PresentCall {
    pub stats: Vec<FileStats>,
    pub config_format: String,
}

impl RecordingPresenter {
    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn calls(&self) -> Vec<PresentCall> {
        self.calls.lock().unwrap().clone()
    }

    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    pub fn was_called(&self) -> bool {
        self.call_count() > 0
    }

    pub fn last_call(&self) -> Option<PresentCall> {
        self.calls.lock().unwrap().last().cloned()
    }
}

impl Default for RecordingPresenter {
    fn default() -> Self {
        Self::new()
    }
}

impl FileStatisticsPresenter for RecordingPresenter {
    fn present(&self, stats: &[FileStats], config: &Config) -> Result<()> {
        self.calls.lock().unwrap().push(PresentCall {
            stats: stats.to_vec(),
            config_format: format!("{:?}", config.format),
        });
        Ok(())
    }
}

// ============================================================================
// RecordingNotifier
// ============================================================================

#[derive(Clone)]
pub struct RecordingNotifier {
    infos: Arc<Mutex<Vec<String>>>,
    warnings: Arc<Mutex<Vec<String>>>,
}

impl RecordingNotifier {
    pub fn new() -> Self {
        Self {
            infos: Arc::new(Mutex::new(Vec::new())),
            warnings: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn info_messages(&self) -> Vec<String> {
        self.infos.lock().unwrap().clone()
    }

    pub fn warning_messages(&self) -> Vec<String> {
        self.warnings.lock().unwrap().clone()
    }

    pub fn has_info(&self, msg: &str) -> bool {
        self.infos.lock().unwrap().iter().any(|m| m.contains(msg))
    }

    pub fn has_warning(&self, msg: &str) -> bool {
        self.warnings
            .lock()
            .unwrap()
            .iter()
            .any(|m| m.contains(msg))
    }

    pub fn info_count(&self) -> usize {
        self.infos.lock().unwrap().len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.lock().unwrap().len()
    }
}

impl Default for RecordingNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisNotifier for RecordingNotifier {
    fn info(&self, message: &str) {
        self.infos.lock().unwrap().push(message.to_string());
    }

    fn warn(&self, message: &str) {
        self.warnings.lock().unwrap().push(message.to_string());
    }
}

// ============================================================================
// MockSnapshotComparator
// ============================================================================

pub struct MockSnapshotComparator {
    result: String,
    should_fail: bool,
}

impl MockSnapshotComparator {
    pub fn new(result: impl Into<String>) -> Self {
        Self {
            result: result.into(),
            should_fail: false,
        }
    }

    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

impl SnapshotComparator for MockSnapshotComparator {
    fn compare(&self, _old: &Path, _new: &Path) -> Result<String> {
        if self.should_fail {
            Err(InfrastructureError::SerializationError {
                format: "JSON".to_string(),
                details: "mock comparison failure".to_string(),
            }
            .into())
        } else {
            Ok(self.result.clone())
        }
    }
}

// ============================================================================
// Test Helpers
// ============================================================================

/// テスト用のモックセットアップ
pub struct MockSetup {
    pub provider: MockFileEntryProvider,
    pub processor: MockFileStatisticsProcessor,
    pub presenter: RecordingPresenter,
    pub notifier: RecordingNotifier,
}

impl MockSetup {
    pub fn new() -> Self {
        Self {
            provider: MockFileEntryProvider::empty(),
            processor: MockFileStatisticsProcessor::empty(),
            presenter: RecordingPresenter::new(),
            notifier: RecordingNotifier::new(),
        }
    }

    pub fn with_entries(mut self, entries: Vec<FileEntry>) -> Self {
        self.provider = MockFileEntryProvider::new(entries);
        self
    }

    pub fn with_stats(mut self, stats: Vec<FileStats>) -> Self {
        self.processor = MockFileStatisticsProcessor::success(stats);
        self
    }

    pub fn with_processor_failure(mut self, msg: impl Into<String>) -> Self {
        self.processor = MockFileStatisticsProcessor::failure(msg);
        self
    }

    pub fn with_provider_failure(mut self) -> Self {
        self.provider = self.provider.with_failure();
        self
    }
}

impl Default for MockSetup {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::{
        super::builders::{ConfigBuilder, FileStatsBuilder},
        *,
    };

    #[test]
    fn recording_presenter_tracks_calls() {
        let presenter = RecordingPresenter::new();
        assert!(!presenter.was_called());
        assert_eq!(presenter.call_count(), 0);

        let stats = [FileStatsBuilder::new("test.rs").lines(10).build()];
        let config = ConfigBuilder::new().build();

        presenter.present(&stats, &config).unwrap();

        assert!(presenter.was_called());
        assert_eq!(presenter.call_count(), 1);

        let last = presenter.last_call().unwrap();
        assert_eq!(last.stats.len(), 1);
    }

    #[test]
    fn recording_notifier_tracks_messages() {
        let notifier = RecordingNotifier::new();

        notifier.info("info message");
        notifier.warn("warning message");

        assert_eq!(notifier.info_count(), 1);
        assert_eq!(notifier.warning_count(), 1);
        assert!(notifier.has_info("info"));
        assert!(notifier.has_warning("warning"));
    }

    #[test]
    fn mock_processor_success() {
        let stats = [FileStatsBuilder::new("test.rs").lines(42).build()];
        let processor = MockFileStatisticsProcessor::success(stats.to_vec());
        let config = ConfigBuilder::new().build();

        let result = processor.measure(vec![], &config).unwrap();
        assert_eq!(result.stats.len(), 1);
        assert_eq!(result.stats[0].lines, 42);
    }

    #[test]
    fn mock_processor_failure() {
        let processor = MockFileStatisticsProcessor::failure("boom");
        let config = ConfigBuilder::new().build();

        let result = processor.measure(vec![], &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("boom"));
    }

    #[test]
    fn mock_setup_builder() {
        let stats = [FileStatsBuilder::new("a.rs").lines(10).build()];
        let setup = MockSetup::new().with_stats(stats.to_vec());

        let outcome = setup
            .processor
            .measure(vec![], &ConfigBuilder::new().build())
            .unwrap();
        assert_eq!(outcome.stats.len(), 1);
    }
}
