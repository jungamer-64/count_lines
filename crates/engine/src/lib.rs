// crates/engine/src/lib.rs
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
use crate::stats::RunResult;

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
    let (tx, rx) = crossbeam_channel::unbounded();
    let (err_tx, err_rx) = std::sync::mpsc::channel();

    let walk_cfg = config.walk.clone();
    let filter_cfg = config.filter.clone();
    let config_inner = config.clone();

    std::thread::spawn(move || {
        let tx = tx.clone();
        let config = config_inner;
        if let Err(e) = crate::filesystem::walk_parallel(&walk_cfg, &filter_cfg, move |path, meta| {
            let res = processor::process_file((path, meta), &config);
            let _ = tx.send(res);
        }) {
            let _ = err_tx.send(e);
        }
    });

    let mut result = RunResult::default();

    for res in rx {
        match res {
            Ok(stats) => result.stats.push(stats),
            Err(e) => {
                if config.strict {
                    return Err(e);
                }
                let path = match &e {
                    EngineError::FileRead { path, .. } => path.clone(),
                    _ => PathBuf::from("<unknown>"),
                };
                result.errors.push((path, e));
            }
        }
    }

    if let Ok(walk_err) = err_rx.try_recv() {
        if config.strict {
            return Err(walk_err);
        }
        result.errors.push((PathBuf::from("<walk>"), walk_err));
    }

    Ok(result)
}
