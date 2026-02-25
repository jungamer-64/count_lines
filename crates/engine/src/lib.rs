// crates/engine/src/lib.rs
use rayon::prelude::*;
use std::path::PathBuf;

pub mod config;
pub mod error;
pub mod filesystem;
pub mod options;
pub mod path_security;
pub mod processor;
pub mod stats;
pub mod watch;

use crate::config::Config;
use crate::error::{EngineError, Result};
use crate::stats::{FileStats, RunResult};

/// Run the file counting engine.
///
/// Returns a `RunResult` containing both successfully processed file statistics
/// and any errors encountered during processing.
///
/// # Errors
///
/// Returns an error only for critical failures (e.g., walk initialization).
/// Individual file processing errors are collected in `RunResult::errors`.
///
/// # Panics
///
/// Panics if the partition results contain unexpected `Ok`/`Err` variants (should never happen).
pub fn run(config: &Config) -> Result<RunResult> {
    let (tx, rx) = crossbeam_channel::bounded(1024);
    let (err_tx, err_rx) = std::sync::mpsc::channel();

    let walk_cfg = config.walk.clone();
    let filter_cfg = config.filter.clone();

    std::thread::spawn(move || {
        if let Err(e) = crate::filesystem::walk_parallel(&walk_cfg, &filter_cfg, &tx) {
            let _ = err_tx.send(e);
        }
    });

    let iter = rx.into_iter().par_bridge();

    let mut result = if config.strict {
        // Strict mode: fail on first error
        let stats = iter
            .map(|item| processor::process_file(item, config))
            .collect::<Result<Vec<_>>>()?;
        RunResult {
            stats,
            errors: Vec::new(),
        }
    } else {
        // Non-strict mode: collect errors alongside successful results
        #[allow(clippy::redundant_closure_for_method_calls)]
        let (results, errors): (Vec<_>, Vec<_>) = iter
            .map(|item| {
                let path = item.0.clone();
                processor::process_file(item, config).map_err(|e| (path, e))
            })
            .partition(|r| r.is_ok());

        let stats: Vec<FileStats> = results.into_iter().map(|r| r.unwrap()).collect();
        let errors: Vec<(PathBuf, EngineError)> =
            errors.into_iter().map(|r| r.unwrap_err()).collect();

        RunResult { stats, errors }
    };

    // Check for walk errors that occurred in the background thread
    if let Ok(walk_err) = err_rx.try_recv() {
        if config.strict {
            return Err(walk_err);
        }
        result.errors.push((PathBuf::from("<walk>"), walk_err));
    }

    Ok(result)
}
