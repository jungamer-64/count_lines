use super::{
    AnalysisNotifier, FileEntryProvider, FileStatisticsPresenter, FileStatisticsProcessor,
};
use crate::domain::{analytics, config::Config};
use anyhow::{Context, Result};

pub struct RunAnalysisCommand<'a> {
    entries: &'a dyn FileEntryProvider,
    processor: &'a dyn FileStatisticsProcessor,
    presenter: &'a dyn FileStatisticsPresenter,
    notifier: Option<&'a dyn AnalysisNotifier>,
}

impl<'a> RunAnalysisCommand<'a> {
    pub fn new(
        entries: &'a dyn FileEntryProvider,
        processor: &'a dyn FileStatisticsProcessor,
        presenter: &'a dyn FileStatisticsPresenter,
        notifier: Option<&'a dyn AnalysisNotifier>,
    ) -> Self {
        Self {
            entries,
            processor,
            presenter,
            notifier,
        }
    }

    pub fn execute(&self, config: &Config) -> Result<()> {
        if config.progress {
            if let Some(notifier) = self.notifier {
                notifier.info("[count_lines] scanning & measuring...");
            }
        }

        let entries = self
            .entries
            .collect(config)
            .context("failed to discover input files")?;
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
        self.presenter
            .present(&stats, config)
            .context("failed to emit output")
    }
}
