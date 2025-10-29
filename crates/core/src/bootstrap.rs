use anyhow::{Context, Result};
use atty::Stream;

use crate::{
    application::commands::{RunAnalysisCommand, RunAnalysisHandler, SnapshotComparator},
    domain::{config::Config, options::OutputFormat},
    infrastructure::adapters::{
        ConsoleNotifier, FileSystemEntryProvider, OutputEmitter, ParallelFileStatisticsProcessor,
        SnapshotDiffAdapter,
    },
    presentation::cli,
};

pub fn run() -> Result<()> {
    let config = cli::load_config().map_err(anyhow::Error::from)?;
    run_with_config(config)
}

pub fn run_with_config(config: Config) -> Result<()> {
    if let Some((old, new)) = &config.compare {
        let comparator = SnapshotDiffAdapter;
        let diff = comparator.compare(old, new).context("compare failed")?;
        println!("{diff}");
        return Ok(());
    }

    if !matches!(config.format, OutputFormat::Json) && atty::is(Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", crate::VERSION, config.jobs);
    }

    let entry_provider = FileSystemEntryProvider;
    let processor = ParallelFileStatisticsProcessor;
    let presenter = OutputEmitter;
    let notifier = ConsoleNotifier;
    let handler = RunAnalysisHandler::new(&entry_provider, &processor, &presenter, Some(&notifier));
    let command = RunAnalysisCommand::new(&config);

    handler.handle(&command)?;
    Ok(())
}
