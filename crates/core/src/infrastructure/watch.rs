use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};

use crate::{domain::config::Config, error::InfrastructureError};

pub struct WatchService;

impl WatchService {
    pub fn run<F>(config: &Config, interval: Duration, mut on_change: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        match Self::watch_with_notify(config, interval, &mut on_change) {
            Ok(()) => Ok(()),
            Err(err) => {
                eprintln!(
                    "[warn] file watcher unavailable ({}). Falling back to polling every {:?}.",
                    err, interval
                );
                Self::poll_loop(interval, &mut on_change)
            }
        }
    }

    #[allow(unreachable_code)]
    fn watch_with_notify<F>(config: &Config, interval: Duration, on_change: &mut F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        let (tx, rx) = mpsc::channel();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(|err| InfrastructureError::OutputError(err.to_string()))?;

        for path in &config.paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|err| InfrastructureError::OutputError(err.to_string()))?;
        }

        let mut pending: Option<Instant> = None;

        loop {
            if let Some(start) = pending {
                let elapsed = start.elapsed();
                if elapsed >= interval {
                    on_change()?;
                    pending = None;
                    continue;
                }
                match rx.recv_timeout(interval - elapsed) {
                    Ok(Ok(event)) => {
                        if Self::is_relevant(&event.kind) {
                            pending = Some(Instant::now());
                        }
                    }
                    Ok(Err(err)) => {
                        eprintln!("[warn] watcher error: {err}");
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        on_change()?;
                        pending = None;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Self::poll_loop(interval, on_change);
                    }
                }
            } else {
                match rx.recv() {
                    Ok(Ok(event)) => {
                        if Self::is_relevant(&event.kind) {
                            pending = Some(Instant::now());
                        }
                    }
                    Ok(Err(err)) => eprintln!("[warn] watcher error: {err}"),
                    Err(_) => return Self::poll_loop(interval, on_change),
                }
            }
        }

        Ok(())
    }

    fn poll_loop<F>(interval: Duration, on_change: &mut F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        loop {
            thread::sleep(interval);
            on_change()?;
        }
    }

    fn is_relevant(kind: &EventKind) -> bool {
        matches!(
            kind,
            EventKind::Any
                | EventKind::Create(_)
                | EventKind::Modify(_)
                | EventKind::Remove(_)
                | EventKind::Other
        )
    }
}
