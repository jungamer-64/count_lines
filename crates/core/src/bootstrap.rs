use std::time::Instant;

use anyhow::{Context, Result, anyhow};
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

/// Entrypoint wrapper that loads CLI configuration and runs the application.
///
/// # Errors
///
/// Returns an error if configuration loading or runtime execution fails.
pub fn run() -> Result<()> {
    let config = cli::load_config().map_err(anyhow::Error::from)?;
    run_with_config(config)
}

/// Run the application with a pre-loaded `Config`.
///
/// # Errors
///
/// Returns an error if configuration validation or execution fails.
pub fn run_with_config(mut config: Config) -> Result<()> {
    validate_config(&config)?;

    if should_exit_after_clearing_cache(&config)? {
        return Ok(());
    }

    if config.watch {
        // watch モードではインクリメンタルキャッシュを強制
        config.incremental = true;
        start_watch_loop(&config)?;
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

    let display_banner = should_display_banner(config, show_banner);

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
    println!("{payload}");
    } else {
        let presenter = OutputEmitter;
        let handler = RunAnalysisHandler::new(&entry_provider, &processor, &presenter, Some(&notifier));
        handler.handle(&command)?;
    }
    Ok(())
}

fn should_display_banner(config: &Config, show_banner: bool) -> bool {
    show_banner
        && config.watch_output != WatchOutput::Jsonl
        && !matches!(config.format, OutputFormat::Json)
        && is_stdout_tty()
}

fn should_exit_after_clearing_cache(config: &Config) -> Result<bool> {
    if config.clear_cache {
        CacheStore::clear(config)?;
        Ok(!config.watch)
    } else {
        Ok(false)
    }
}

fn validate_config(config: &Config) -> Result<()> {
    if config.watch && config.compare.is_some() {
        Err(anyhow!("--compare cannot be used together with --watch"))
    } else {
        Ok(())
    }
}

fn start_watch_loop(config: &Config) -> Result<()> {
    run_analysis(config, true)?;
    WatchService::run(config, config.watch_interval, || run_analysis(config, false))?;
    Ok(())
}

    // Use libc directly to avoid bringing the `atty` crate into the dependency tree.
    // This performs a simple isatty(STDOUT_FILENO) check on Unix-like platforms.
    //
    // Semgrep may flag the following `unsafe` usage. We document the safety
    // rationale below and add an inline suppression comment so automated
    // scanners can be satisfied while keeping the implementation minimal and
    // dependency-free.
    //
    // Use `nix::unistd::isatty` which provides a safe wrapper around the
    // platform `isatty` check. This lets us avoid any `unsafe` blocks in the
    // repository source code while still correctly detecting terminal
    // connectivity on Unix-like platforms.
    fn is_stdout_tty() -> bool {
        #[cfg(unix)]
        {
            // Use the std `Stdout` handle which implements `AsFd` so we can
            // call the generic `nix::unistd::isatty` safely. If the call
            // fails for any reason, conservatively return `false`.
            nix::unistd::isatty(std::io::stdout()).unwrap_or(false)
        }

        #[cfg(not(unix))]
        {
            // On non-Unix platforms we conservatively return `true` so behavior is preserved.
            true
        }
    }
