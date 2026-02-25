// crates/engine/src/watch.rs
use crate::config::Config;
use crate::error::Result;
use notify::{RecursiveMode, Watcher};
use std::sync::mpsc::channel;

/// Watch files for changes and run the callback.
///
/// This function blocks indefinitely.
pub fn watch_loop<F>(config: &Config, mut on_event: F) -> Result<()>
where
    F: FnMut(),
{
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            let _ = tx.send(event);
        }
        Err(e) => eprintln!("watch error: {e:?}"),
    })?;

    // Add paths to be watched
    for root in &config.walk.roots {
        if root.exists() {
            watcher.watch(root, RecursiveMode::Recursive)?;
        }
    }

    // Initial run
    println!("[count_lines] Starting watch mode...");
    on_event();

    let debounce_interval = config.watch_interval;

    // Loop forever
    loop {
        // Event loop
        while rx.recv().is_ok() {
            // Debounce
            std::thread::sleep(debounce_interval);
            // Drain
            while rx.try_recv().is_ok() {}

            on_event();
        }
    }
}
