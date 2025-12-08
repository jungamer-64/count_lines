// crates/core/src/application/commands/run_analysis/handler.rs
//! RunAnalysisHandlerのリファクタリング版

use std::path::PathBuf;

use crate::{
    application::commands::{
        AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
        MeasurementOutcome, RunAnalysisCommand,
    },
    domain::{analytics::SortStrategy, config::Config},
    error::*,
};

/// 分析実行ハンドラー（改善版）
pub struct RunAnalysisHandler<'a> {
    entries: &'a dyn FileEntryProvider,
    processor: &'a dyn FileStatisticsProcessor,
    presenter: &'a dyn FileStatisticsPresenter,
    notifier: Option<&'a dyn AnalysisNotifier>,
}

impl<'a> RunAnalysisHandler<'a> {
    pub fn new(
        entries: &'a dyn FileEntryProvider,
        processor: &'a dyn FileStatisticsProcessor,
        presenter: &'a dyn FileStatisticsPresenter,
        notifier: Option<&'a dyn AnalysisNotifier>,
    ) -> Self {
        Self { entries, processor, presenter, notifier }
    }

    pub fn handle(&self, command: &RunAnalysisCommand<'_>) -> Result<RunOutcome> {
        let config = command.config();

        self.log_start(config);

        let entries = self.collect_entries(config)?;
        let MeasurementOutcome { stats: mut stats, changed_files, removed_files } =
            self.measure_statistics(entries, config)?;
        self.apply_sorting(&mut stats, config);
        self.present_results(&stats, config)?;

        self.log_completion(&stats);

        Ok(RunOutcome { stats, changed_files, removed_files })
    }

    fn collect_entries(&self, config: &Config) -> Result<Vec<crate::domain::model::FileEntry>> {
        self.entries.collect(config).map_err(|e| ApplicationError::FileCollectionFailed(e.to_string()).into())
    }

    fn measure_statistics(
        &self,
        entries: Vec<crate::domain::model::FileEntry>,
        config: &Config,
    ) -> Result<MeasurementOutcome> {
        match self.processor.measure(entries, config) {
            Ok(outcome) => Ok(outcome),
            Err(err) if config.strict => Err(ApplicationError::MeasurementFailed(err.to_string()).into()),
            Err(err) => {
                self.log_warning(&format!("Measurement warning: {}", err));
                Ok(MeasurementOutcome::new(Vec::new(), Vec::new(), Vec::new()))
            }
        }
    }

    fn apply_sorting(&self, stats: &mut [crate::domain::model::FileStats], config: &Config) {
        if !config.total_only && !config.summary_only && !config.sort_specs.is_empty() {
            let strategy = SortStrategy::from_legacy(config.sort_specs.clone());
            strategy.apply(stats);
        }
    }

    fn present_results(&self, stats: &[crate::domain::model::FileStats], config: &Config) -> Result<()> {
        self.presenter
            .present(stats, config)
            .map_err(|e| ApplicationError::PresentationFailed(e.to_string()).into())
    }

    fn log_start(&self, config: &Config) {
        if config.progress
            && let Some(notifier) = self.notifier
        {
            notifier.info("[count_lines] Starting analysis...");
        }
    }

    fn log_completion(&self, stats: &[crate::domain::model::FileStats]) {
        if let Some(notifier) = self.notifier {
            notifier.info(&format!("[count_lines] Completed: {} files processed", stats.len()));
        }
    }

    fn log_warning(&self, message: &str) {
        if let Some(notifier) = self.notifier {
            notifier.warn(message);
        }
    }
}

#[derive(Debug)]
pub struct RunOutcome {
    pub stats: Vec<crate::domain::model::FileStats>,
    pub changed_files: Vec<PathBuf>,
    pub removed_files: Vec<PathBuf>,
}

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        sync::{Arc, Mutex},
        time::Duration,
    };

    use super::*;
    use crate::{
        application::commands::{
            AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
        },
        domain::{
            config::{Config, value_objects::Filters},
            model::{FileEntry, FileStats, FileStatsBuilder},
            options::{OutputFormat, SortKey, WatchOutput},
            value_objects::{CharCount, FileExtension, FileName, FilePath, FileSize, LineCount},
        },
        error::{ApplicationError, CountLinesError, InfrastructureError, Result},
    };

    #[derive(Clone)]
    struct RecordingPresenter {
        calls: Arc<Mutex<Vec<Vec<FileStats>>>>,
    }

    impl RecordingPresenter {
        fn new() -> Self {
            Self { calls: Arc::new(Mutex::new(Vec::new())) }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().unwrap().len()
        }

        fn last_stats(&self) -> Option<Vec<FileStats>> {
            self.calls.lock().unwrap().last().cloned()
        }
    }

    impl Default for RecordingPresenter {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FileStatisticsPresenter for RecordingPresenter {
        fn present(&self, stats: &[FileStats], _config: &Config) -> Result<()> {
            self.calls.lock().unwrap().push(stats.to_vec());
            Ok(())
        }
    }

    struct FailingPresenter;

    impl FileStatisticsPresenter for FailingPresenter {
        fn present(&self, _stats: &[FileStats], _config: &Config) -> Result<()> {
            Err(InfrastructureError::OutputError("failed to present".into()).into())
        }
    }

    #[derive(Clone)]
    struct RecordingNotifier {
        infos: Arc<Mutex<Vec<String>>>,
        warnings: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingNotifier {
        fn new() -> Self {
            Self { infos: Arc::new(Mutex::new(Vec::new())), warnings: Arc::new(Mutex::new(Vec::new())) }
        }

        fn info_messages(&self) -> Vec<String> {
            self.infos.lock().unwrap().clone()
        }

        fn warning_messages(&self) -> Vec<String> {
            self.warnings.lock().unwrap().clone()
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

    struct StubProvider {
        entries: Vec<FileEntry>,
        fail: bool,
    }

    impl StubProvider {
        fn success(entries: Vec<FileEntry>) -> Self {
            Self { entries, fail: false }
        }

        fn failure() -> Self {
            Self { entries: Vec::new(), fail: true }
        }
    }

    impl FileEntryProvider for StubProvider {
        fn collect(&self, _config: &Config) -> Result<Vec<FileEntry>> {
            if self.fail {
                Err(InfrastructureError::FileSystemOperation {
                    operation: "collect".to_string(),
                    path: PathBuf::from("."),
                    source: std::io::Error::new(std::io::ErrorKind::Other, "mock failure"),
                }
                .into())
            } else {
                Ok(self.entries.clone())
            }
        }
    }

    enum ProcessorMode {
        Success(MeasurementOutcome),
        Failure(String),
    }

    struct RecordingProcessor {
        mode: ProcessorMode,
        received_entries: Arc<Mutex<Vec<FileEntry>>>,
    }

    impl RecordingProcessor {
        fn success(stats: Vec<FileStats>) -> Self {
            let outcome = MeasurementOutcome::new(stats, Vec::new(), Vec::new());
            Self { mode: ProcessorMode::Success(outcome), received_entries: Arc::new(Mutex::new(Vec::new())) }
        }

        fn failure(message: impl Into<String>) -> Self {
            Self {
                mode: ProcessorMode::Failure(message.into()),
                received_entries: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn received(&self) -> Vec<FileEntry> {
            self.received_entries.lock().unwrap().clone()
        }
    }

    impl FileStatisticsProcessor for RecordingProcessor {
        fn measure(&self, entries: Vec<FileEntry>, _config: &Config) -> Result<MeasurementOutcome> {
            let mut guard = self.received_entries.lock().unwrap();
            *guard = entries.clone();
            drop(guard);

            match &self.mode {
                ProcessorMode::Success(outcome) => Ok(outcome.clone()),
                ProcessorMode::Failure(message) => Err(InfrastructureError::MeasurementError {
                    path: PathBuf::from("mock"),
                    reason: message.clone(),
                }
                .into()),
            }
        }
    }

    fn base_config() -> Config {
        Config {
            format: OutputFormat::Table,
            sort_specs: vec![],
            top_n: None,
            by_modes: vec![],
            summary_only: false,
            total_only: false,
            by_limit: None,
            filters: Filters::default(),
            hidden: false,
            follow: false,
            use_git: false,
            case_insensitive_dedup: false,
            respect_gitignore: true,
            use_ignore_overrides: false,
            jobs: 1,
            no_default_prune: false,
            max_depth: None,
            enumerator_threads: None,
            abs_path: false,
            abs_canonical: false,
            trim_root: None,
            words: false,
            sloc: false,
            count_newlines_in_chars: false,
            text_only: false,
            fast_text_detect: false,
            files_from: None,
            files_from0: None,
            paths: vec![PathBuf::from(".")],
            mtime_since: None,
            mtime_until: None,
            total_row: false,
            progress: false,
            ratio: false,
            output: None,
            strict: false,
            incremental: false,
            cache_dir: None,
            cache_verify: false,
            clear_cache: false,
            watch: false,
            watch_interval: Duration::from_secs(1),
            watch_output: WatchOutput::Full,
            compare: None,
        }
    }

    fn make_entry(name: &str) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            meta: crate::domain::value_objects::FileMeta {
                size: 0,
                mtime: None,
                is_text: true,
                ext: "rs".to_string(),
                name: name.to_string(),
            },
        }
    }

    fn make_stats(name: &str, lines: usize) -> FileStats {
        FileStatsBuilder::new(FilePath::new(PathBuf::from(name)))
            .lines(LineCount::new(lines))
            .chars(CharCount::new(lines * 10))
            .ext(FileExtension::new("rs".to_string()))
            .name(FileName::new(name.to_string()))
            .size(FileSize::new(0))
            .build()
            .to_legacy()
    }

    #[test]
    fn handler_sorts_stats_and_notifies_on_success() {
        let mut config = base_config();
        config.progress = true;
        config.sort_specs = vec![(SortKey::Lines, true)];

        let entries = vec![make_entry("a.rs"), make_entry("b.rs")];
        let stats = vec![make_stats("a.rs", 10), make_stats("b.rs", 30)];

        let provider = StubProvider::success(entries.clone());
        let processor = RecordingProcessor::success(stats);
        let presenter = RecordingPresenter::new();
        let notifier = RecordingNotifier::new();

        let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, Some(&notifier));
        let command = RunAnalysisCommand::new(&config);
        let outcome = handler.handle(&command).expect("analysis succeeds");

        let received = processor.received();
        assert_eq!(received.len(), entries.len());

        assert_eq!(outcome.stats.len(), 2);
        assert_eq!(outcome.stats[0].lines, 30);
        assert_eq!(outcome.stats[1].lines, 10);

        assert_eq!(presenter.call_count(), 1);
        let presented = presenter.last_stats().expect("presenter was called");
        let presented_lines: Vec<_> = presented.iter().map(|s| s.lines).collect();
        assert_eq!(presented_lines, vec![30, 10]);

        let info_messages = notifier.info_messages();
        assert_eq!(info_messages.len(), 2, "start and completion messages should be emitted");
        assert!(info_messages[0].contains("Starting analysis"));
        assert!(info_messages[1].contains("Completed"));
    }

    #[test]
    fn strict_mode_bubbles_up_measurement_failures() {
        let mut config = base_config();
        config.strict = true;

        let provider = StubProvider::success(vec![make_entry("main.rs")]);
        let processor = RecordingProcessor::failure("boom");
        let presenter = RecordingPresenter::new();

        let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, None);
        let command = RunAnalysisCommand::new(&config);
        let err = handler.handle(&command).unwrap_err();

        match err {
            CountLinesError::Application(ApplicationError::MeasurementFailed(message)) => {
                assert!(message.contains("boom"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn non_strict_mode_emits_warning_and_returns_empty_stats() {
        let mut config = base_config();
        config.strict = false;

        let provider = StubProvider::success(vec![make_entry("lib.rs")]);
        let processor = RecordingProcessor::failure("temporary failure");
        let presenter = RecordingPresenter::new();
        let notifier = RecordingNotifier::new();

        let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, Some(&notifier));
        let command = RunAnalysisCommand::new(&config);

        let outcome = handler.handle(&command).expect("handler should recover when strict mode is disabled");
        assert!(outcome.stats.is_empty(), "stats should be empty when measurement failed in non-strict mode");
        assert!(outcome.changed_files.is_empty());
        assert!(outcome.removed_files.is_empty());

        assert_eq!(presenter.call_count(), 1);
        let warnings = notifier.warning_messages();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("Measurement warning"));
    }

    #[test]
    fn provider_failure_is_translated_to_application_error() {
        let config = base_config();
        let provider = StubProvider::failure();
        let processor = RecordingProcessor::success(vec![]);
        let presenter = RecordingPresenter::new();

        let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, None);
        let command = RunAnalysisCommand::new(&config);

        let err = handler.handle(&command).unwrap_err();
        match err {
            CountLinesError::Application(ApplicationError::FileCollectionFailed(message)) => {
                assert!(message.contains("collect"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    #[test]
    fn presenter_failure_is_translated_to_application_error() {
        let config = base_config();
        let entries = vec![make_entry("a.rs")];
        let stats = vec![make_stats("a.rs", 10)];

        let provider = StubProvider::success(entries);
        let processor = RecordingProcessor::success(stats);
        let presenter = FailingPresenter;

        let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, None);
        let command = RunAnalysisCommand::new(&config);

        let err = handler.handle(&command).unwrap_err();
        match err {
            CountLinesError::Application(ApplicationError::PresentationFailed(message)) => {
                assert!(message.contains("failed to present"));
            }
            other => panic!("unexpected error variant: {other:?}"),
        }
    }
}
