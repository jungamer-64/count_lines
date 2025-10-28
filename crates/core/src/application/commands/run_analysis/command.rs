use crate::domain::config::Config;

/// Command payload encapsulating the configuration for running an analysis.
#[derive(Debug, Clone, Copy)]
pub struct RunAnalysisCommand<'a> {
    config: &'a Config,
}

impl<'a> RunAnalysisCommand<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &'a Config {
        self.config
    }
}
