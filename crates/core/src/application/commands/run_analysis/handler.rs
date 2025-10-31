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
        let MeasurementOutcome { stats: mut_stats, changed_files, removed_files } =
            self.measure_statistics(entries, config)?;
        let mut stats = mut_stats;
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
