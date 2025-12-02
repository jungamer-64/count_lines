// tests/unit/application/commands_test.rs
use std::{
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use count_lines_core::{
    application::commands::{
        AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
        MeasurementOutcome, RunAnalysisCommand, RunAnalysisHandler,
    },
    domain::{
        config::{ByKey, Config, Filters},
        model::{FileEntry, FileMeta, FileStats},
        options::{OutputFormat, SortKey},
    },
    error::{ApplicationError, Result},
};

fn base_config() -> Config {
    Config {
        format: OutputFormat::Json,
        sort_specs: Vec::new(),
        top_n: None,
        by_modes: vec![ByKey::Ext],
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
        jobs: 2,
        no_default_prune: false,
        max_depth: None,
        enumerator_threads: None,
        abs_path: false,
        abs_canonical: false,
        trim_root: None,
        words: false,
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
        watch_output: count_lines_core::domain::options::WatchOutput::Full,
        compare: None,
        sloc: false,
    }
}

fn make_entry(path: &str) -> FileEntry {
    FileEntry {
        path: PathBuf::from(path),
        meta: FileMeta { size: 0, mtime: None, is_text: true, ext: "rs".into(), name: "stub.rs".into() },
    }
}

fn make_stats(path: &str, lines: usize) -> FileStats {
    let meta = FileMeta {
        size: 0,
        mtime: None,
        is_text: true,
        ext: "rs".into(),
        name: PathBuf::from(path).file_name().unwrap().to_string_lossy().into(),
    };
    FileStats::new(PathBuf::from(path), lines, lines * 10, Some(lines / 2), &meta)
}

struct StubProvider {
    entries: Vec<FileEntry>,
}

impl FileEntryProvider for StubProvider {
    fn collect(&self, _config: &Config) -> Result<Vec<FileEntry>> {
        Ok(self.entries.clone())
    }
}

#[derive(Clone)]
enum ProcessorOutcome {
    Success(MeasurementOutcome),
    Failure(String),
}

struct StubProcessor {
    outcome: ProcessorOutcome,
}

impl StubProcessor {
    fn success(stats: Vec<FileStats>) -> Self {
        Self { outcome: ProcessorOutcome::Success(MeasurementOutcome::new(stats, Vec::new(), Vec::new())) }
    }

    fn failure(message: &str) -> Self {
        Self { outcome: ProcessorOutcome::Failure(message.into()) }
    }

    fn with_outcome(outcome: MeasurementOutcome) -> Self {
        Self { outcome: ProcessorOutcome::Success(outcome) }
    }
}

impl FileStatisticsProcessor for StubProcessor {
    fn measure(&self, _entries: Vec<FileEntry>, _config: &Config) -> Result<MeasurementOutcome> {
        match &self.outcome {
            ProcessorOutcome::Success(outcome) => Ok(outcome.clone()),
            ProcessorOutcome::Failure(message) => {
                Err(ApplicationError::MeasurementFailed(message.clone()).into())
            }
        }
    }
}

struct RecordingPresenter {
    calls: Arc<Mutex<Vec<Vec<(String, usize)>>>>,
}

impl RecordingPresenter {
    fn new() -> (Self, Arc<Mutex<Vec<Vec<(String, usize)>>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        (Self { calls: Arc::clone(&calls) }, calls)
    }
}

impl FileStatisticsPresenter for RecordingPresenter {
    fn present(&self, stats: &[FileStats], _config: &Config) -> Result<()> {
        let snapshot =
            stats.iter().map(|s| (s.path.to_string_lossy().to_string(), s.lines)).collect::<Vec<_>>();
        self.calls.lock().unwrap().push(snapshot);
        Ok(())
    }
}

#[derive(Default)]
struct RecordingNotifier {
    infos: Arc<Mutex<Vec<String>>>,
    warns: Arc<Mutex<Vec<String>>>,
}

impl RecordingNotifier {
    fn new() -> (Self, Arc<Mutex<Vec<String>>>, Arc<Mutex<Vec<String>>>) {
        let infos = Arc::new(Mutex::new(Vec::new()));
        let warns = Arc::new(Mutex::new(Vec::new()));
        (Self { infos: Arc::clone(&infos), warns: Arc::clone(&warns) }, infos, warns)
    }
}

impl AnalysisNotifier for RecordingNotifier {
    fn info(&self, message: &str) {
        self.infos.lock().unwrap().push(message.to_string());
    }

    fn warn(&self, message: &str) {
        self.warns.lock().unwrap().push(message.to_string());
    }
}

#[test]
fn run_analysis_command_exposes_config_reference() {
    let config = base_config();
    let command = RunAnalysisCommand::new(&config);
    assert!(std::ptr::eq(command.config(), &config));
}

#[test]
fn handler_sorts_results_and_notifies_progress() {
    let mut config = base_config();
    config.sort_specs = vec![(SortKey::Lines, true)];
    config.progress = true;

    let provider = StubProvider { entries: vec![make_entry("src/lib.rs")] };
    let processor = StubProcessor::success(vec![make_stats("b.rs", 4), make_stats("a.rs", 12)]);
    let (presenter, calls) = RecordingPresenter::new();
    let (notifier, infos, _) = RecordingNotifier::new();

    let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, Some(&notifier));
    let outcome = handler.handle(&RunAnalysisCommand::new(&config)).expect("handler succeeds");
    assert_eq!(outcome.changed_files.len(), 0);

    let recorded = calls.lock().unwrap();
    let first_call = recorded.first().expect("presenter called once");
    assert_eq!(first_call, &vec![("a.rs".into(), 12), ("b.rs".into(), 4)]);

    let info_messages = infos.lock().unwrap();
    assert!(info_messages.iter().any(|msg| msg.contains("Starting analysis")));
}

#[test]
fn handler_propagates_errors_when_strict() {
    let mut config = base_config();
    config.strict = true;

    let provider = StubProvider { entries: vec![make_entry("src/lib.rs")] };
    let processor = StubProcessor::failure("boom");
    let (presenter, calls) = RecordingPresenter::new();

    let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, None);
    let err = handler.handle(&RunAnalysisCommand::new(&config)).expect_err("strict mode should fail");
    assert!(err.to_string().contains("Failed to measure file statistics"));
    let mut current: Option<&dyn std::error::Error> = err.source();
    let mut found = false;
    while let Some(cause) = current {
        if cause.to_string().contains("boom") {
            found = true;
            break;
        }
        current = cause.source();
    }
    assert!(found, "expected error chain to contain source with 'boom'");
    assert!(calls.lock().unwrap().is_empty());
}

#[test]
fn handler_warns_and_continues_when_not_strict() {
    let config = base_config();
    let provider = StubProvider { entries: vec![make_entry("src/lib.rs")] };
    let processor = StubProcessor::failure("incomplete data");
    let (presenter, calls) = RecordingPresenter::new();
    let (notifier, _, warns) = RecordingNotifier::new();

    let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, Some(&notifier));
    let outcome = handler.handle(&RunAnalysisCommand::new(&config)).expect("non-strict mode succeeds");
    assert!(outcome.stats.is_empty());

    let warnings = warns.lock().unwrap();
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].contains("incomplete data"));

    let recorded = calls.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert!(recorded[0].is_empty());
}

#[test]
fn handler_returns_change_metadata() {
    let mut config = base_config();
    config.incremental = true;

    let provider = StubProvider { entries: vec![make_entry("src/lib.rs")] };
    let measurement = MeasurementOutcome::new(
        vec![make_stats("src/lib.rs", 4)],
        vec![PathBuf::from("src/lib.rs")],
        vec![PathBuf::from("old.rs")],
    );
    let processor = StubProcessor::with_outcome(measurement);
    let (presenter, _) = RecordingPresenter::new();

    let handler = RunAnalysisHandler::new(&provider, &processor, &presenter, None);
    let outcome = handler.handle(&RunAnalysisCommand::new(&config)).expect("handler succeeds");

    assert_eq!(outcome.changed_files, vec![PathBuf::from("src/lib.rs")]);
    assert_eq!(outcome.removed_files, vec![PathBuf::from("old.rs")]);
}
