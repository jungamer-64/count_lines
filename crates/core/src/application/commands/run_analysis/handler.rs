use anyhow::{Context, Result};

use super::{
    super::ports::{AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor},
    command::RunAnalysisCommand,
};
use crate::domain::{analytics, config::Config};

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

    pub fn handle(&self, command: &RunAnalysisCommand<'_>) -> Result<()> {
        self.execute(command.config())
    }

    fn execute(&self, config: &Config) -> Result<()> {
        if config.progress
            && let Some(notifier) = self.notifier
        {
            notifier.info("[count_lines] scanning & measuring...");
        }

        let entries = self.entries.collect(config).context("failed to discover input files")?;
        let mut stats = match self.processor.measure(entries, config) {
            Ok(stats) => stats,
            Err(err) => {
                if config.strict {
                    return Err(err).context("failed to measure entries");
                }
                if let Some(notifier) = self.notifier {
                    notifier.warn(&format!("[warn] {err}"));
                }
                Vec::new()
            }
        };

        analytics::apply_sort(&mut stats, config);
        self.presenter.present(&stats, config).context("failed to emit output")
    }
}
