use crate::application::commands::{RunAnalysisCommand, SnapshotComparator};
use crate::domain::config::Config;
use crate::domain::options::OutputFormat;
use crate::infrastructure::adapters::{
    ConsoleNotifier, FileSystemEntryProvider, OutputEmitter, ParallelFileStatisticsProcessor,
    SnapshotDiffAdapter,
};
use crate::presentation::cli;
use anyhow::{Context, Result};
use atty::Stream;

pub fn run() -> Result<()> {
    let config = cli::load_config()?;
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
    let command = RunAnalysisCommand::new(&entry_provider, &processor, &presenter, Some(&notifier));

    command.execute(&config)
}
