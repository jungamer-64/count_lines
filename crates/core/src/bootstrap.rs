use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use atty::Stream;
use chrono::Utc;
use serde_json::json;

use crate::{
    application::commands::{RunAnalysisCommand, RunAnalysisHandler, SnapshotComparator},
    domain::{
        config::Config,
        model::Summary,
        options::{OutputFormat, WatchOutput},
    },
    infrastructure::{
        adapters::{
            ConsoleNotifier, FileSystemEntryProvider, JsonlWatchEmitter, OutputEmitter,
            ParallelFileStatisticsProcessor, SnapshotDiffAdapter,
        },
        cache::CacheStore,
        watch::WatchService,
    },
    presentation::cli,
};

pub fn run() -> Result<()> {
    let config = cli::load_config().map_err(anyhow::Error::from)?;
    run_with_config(config)
}

pub fn run_with_config(mut config: Config) -> Result<()> {
    if config.watch && config.compare.is_some() {
        return Err(anyhow!("--compare cannot be used together with --watch"));
    }

    if config.clear_cache {
        CacheStore::clear(&config)?;
        if !config.watch {
            return Ok(());
        }
    }

    if config.watch {
        // watch モードではインクリメンタルキャッシュを強制
        config.incremental = true;
        run_analysis(&config, true)?;
        WatchService::run(&config, config.watch_interval, || run_analysis(&config, false))?
    } else {
        run_analysis(&config, true)?;
    }

    Ok(())
}

fn run_analysis(config: &Config, show_banner: bool) -> Result<()> {
    let start = Instant::now();
    if let Some((old, new)) = &config.compare {
        let comparator = SnapshotDiffAdapter;
        let diff = comparator.compare(old, new).context("compare failed")?;
        println!("{diff}");
        return Ok(());
    }

    let display_banner = show_banner
        && config.watch_output != WatchOutput::Jsonl
        && !matches!(config.format, OutputFormat::Json)
        && atty::is(Stream::Stdout);

    if display_banner {
        eprintln!("count_lines v{} · parallel={}", crate::VERSION, config.jobs);
    }

    let entry_provider = FileSystemEntryProvider;
    let processor = ParallelFileStatisticsProcessor;
    let notifier = ConsoleNotifier;
    let command = RunAnalysisCommand::new(config);

    if config.watch && config.watch_output == WatchOutput::Jsonl {
        let presenter = JsonlWatchEmitter::new();
        let handler = RunAnalysisHandler::new(&entry_provider, &processor, &presenter, Some(&notifier));
        let outcome = handler.handle(&command)?;

        let duration_ms = start.elapsed().as_millis();
        let summary = Summary::from_stats(&outcome.stats);
        let changed: Vec<_> = outcome.changed_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let removed: Vec<_> = outcome.removed_files.iter().map(|p| p.to_string_lossy().to_string()).collect();

        let payload = json!({
            "type": "run",
            "status": "ok",
            "timestamp": Utc::now().to_rfc3339(),
            "duration_ms": duration_ms,
            "summary": {
                "lines": summary.lines,
                "chars": summary.chars,
                "words": summary.words,
                "files": summary.files
            },
            "changed_files": changed,
            "removed_files": removed
        });
        println!("{}", payload);
    } else {
        let presenter = OutputEmitter;
        let handler = RunAnalysisHandler::new(&entry_provider, &processor, &presenter, Some(&notifier));
        handler.handle(&command)?;
    }
    Ok(())
}
