use crate::config::Config;
use crate::engine;
use crate::error::Result;
use crate::options::WatchOutput;
use crate::presentation;
use notify::{RecursiveMode, Watcher};
use std::sync::mpsc::channel;

pub fn watch_paths(config: &Config) -> Result<()> {
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            let _ = tx.send(event);
        }
        Err(e) => eprintln!("watch error: {e:?}"),
    })?;

    // Add paths to be watched
    // We watch the roots.
    for root in &config.walk.roots {
        if root.exists() {
            watcher.watch(root, RecursiveMode::Recursive)?;
        }
    }

    // Initial run
    println!("[count_lines] Starting watch mode...");
    run_cycle(config)?;

    let debounce_interval = config.watch_interval;

    // Loop forever
    loop {
        // Event loop
        while rx.recv().is_ok() {
            // Debounce: consume all events in the queue until silence for interval
            std::thread::sleep(debounce_interval);

            // Drain channel to clear pending events
            while rx.try_recv().is_ok() {}

            // Clear screen and re-run
            presentation::print_clear_screen(&config.watch_output);
            run_cycle(config);
        }
    }
}

fn run_cycle(config: &Config) {
    // Clear screen for full output
    if matches!(config.watch_output, WatchOutput::Full) {
        // ANSI clear screen
        print!("\x1B[2J\x1B[1;1H");
    }

    match engine::run(config) {
        Ok(result) => {
            // Print any processing errors to stderr
            for (path, err) in &result.errors {
                eprintln!("Error processing {}: {err}", path.display());
            }

            // Print results
            presentation::print_results(&result.stats, config);
        }
        Err(e) => eprintln!("Error in watch cycle: {e}"),
    }
}
