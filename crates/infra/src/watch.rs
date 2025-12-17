use std::time::Duration;

#[cfg(not(feature = "watch"))]
use count_lines_shared_kernel::InfrastructureError;
use count_lines_shared_kernel::Result;

#[cfg(feature = "watch")]
use {
    count_lines_shared_kernel::InfrastructureError,
    notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind},
    std::{sync::mpsc, thread, time::Instant},
};

use count_lines_domain::config::Config;

#[cfg(feature = "watch")]
pub struct WatchService;

#[cfg(feature = "watch")]
impl WatchService {
    /// Run the watch service: try to use filesystem notifications and fall back to polling.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided `on_change` callback returns an error during polling.
    pub fn run<F>(config: &Config, interval: Duration, mut on_change: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        match Self::watch_with_notify(config, interval, &mut on_change) {
            Ok(()) => Ok(()),
            Err(err) => {
                eprintln!(
                    "[warn] file watcher unavailable ({err}). Falling back to polling every {interval:?}.",
                );
                Self::poll_loop(interval, &mut on_change)
            }
        }
    }

    fn watch_with_notify<F>(config: &Config, interval: Duration, on_change: &mut F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        let (watcher, rx) = Self::create_watcher(config)?;
        // Keep `watcher` alive in this scope so it continues watching.
        let _keep = &watcher;
        Self::event_loop(&rx, interval, on_change)
    }

    fn create_watcher(
        config: &Config,
    ) -> std::result::Result<
        (
            RecommendedWatcher,
            std::sync::mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>,
        ),
        InfrastructureError,
    > {
        let (tx, rx) = mpsc::channel();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })
        .map_err(|err| InfrastructureError::OutputError { message: err.to_string(), source: Some(Box::new(err)) })?;

        for path in &config.paths {
            watcher
                .watch(path, RecursiveMode::Recursive)
                .map_err(|err| InfrastructureError::OutputError { message: err.to_string(), source: Some(Box::new(err)) })?;
        }

        Ok((watcher, rx))
    }

    fn event_loop<F>(
        rx: &std::sync::mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>,
        interval: Duration,
        on_change: &mut F,
    ) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        let mut pending: Option<Instant> = None;

        loop {
            if pending.is_some() {
                Self::process_pending(rx, interval, on_change, &mut pending)?;
            } else {
                Self::process_idle(rx, interval, on_change, &mut pending)?;
            }
        }
    }

    fn process_pending<F>(
        rx: &std::sync::mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>,
        interval: Duration,
        on_change: &mut F,
        pending: &mut Option<Instant>,
    ) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        let start = match pending.take() {
            Some(s) => s,
            None => {
                eprintln!("[warn] process_pending called without pending set; returning");
                return Ok(());
            }
        };
        let elapsed = start.elapsed();
        if elapsed >= interval {
            on_change()?;
            *pending = None;
            return Ok(());
        }

        let remaining = interval.checked_sub(elapsed).unwrap_or_default();
        match rx.recv_timeout(remaining) {
            Ok(Ok(event)) => {
                if Self::is_relevant(event.kind) {
                    *pending = Some(Instant::now());
                }
            }
            Ok(Err(err)) => {
                eprintln!("[warn] watcher error: {err}");
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                on_change()?;
                *pending = None;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Self::poll_loop(interval, on_change);
            }
        }

        Ok(())
    }

    fn process_idle<F>(
        rx: &std::sync::mpsc::Receiver<std::result::Result<notify::Event, notify::Error>>,
        interval: Duration,
        on_change: &mut F,
        pending: &mut Option<Instant>,
    ) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        match rx.recv() {
            Ok(Ok(event)) => {
                if Self::is_relevant(event.kind) {
                    *pending = Some(Instant::now());
                }
            }
            Ok(Err(err)) => eprintln!("[warn] watcher error: {err}"),
            Err(_) => return Self::poll_loop(interval, on_change),
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

    const fn is_relevant(kind: EventKind) -> bool {
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

#[cfg(not(feature = "watch"))]
pub struct WatchService;

#[cfg(not(feature = "watch"))]
impl WatchService {
    pub fn run<F>(_config: &Config, _interval: Duration, _on_change: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        Err(
            InfrastructureError::OutputError { message: "watch feature disabled at compile time".to_string(), source: None }.into(),
        )
    }
}
