// src/main.rs
#![allow(clippy::multiple_crate_versions)]

//! Entry point for the `count_lines` application.
//!
//! This module wires together the command‑line parsing, configuration
//! construction and orchestrates the measurement, sorting and output
//! logic. It also supports an optional comparison mode to diff two
//! JSON snapshots.

use anyhow::{Context, Result};
use atty::Stream;
use clap::Parser;

// Pull module definitions into scope. These declarations allow the
// compiler to locate the code in separate files within `src/`.
pub mod cli;
pub mod compare;
pub mod compute;
pub mod config;
pub mod files;
pub mod output;
pub mod types;
pub mod util;
pub mod version;

// Re‑export the version constant so it can be referenced as
// `crate::VERSION` from other modules, matching the original API.
pub use version::VERSION;

fn main() -> Result<()> {
    // Parse command line arguments using clap's derive API.
    let args = cli::Args::parse();
    // Convert the raw arguments into a richer configuration. Any
    // validation errors will surface here.
    let config = config::Config::try_from(args)?;

    // If the `--compare` option was provided, run the comparison of two
    // JSON snapshot files and print the diff. Exit immediately
    // afterwards since no further processing is required.
    if let Some((old, new)) = &config.compare {
        let diff = compare::run(old, new).context("compare failed")?;
        println!("{}", diff);
        return Ok(());
    }

    // Print a banner when outputting human‑friendly table formats to a
    // terminal. Suppress the banner when writing machine‑readable
    // formats (like JSON/YAML/CSV) or when stdout is redirected.
    if !matches!(config.format, cli::OutputFormat::Json) && atty::is(Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", VERSION, config.jobs);
    }

    // Optionally report progress to stderr. This is kept simple to
    // avoid interfering with the structured output on stdout.
    if config.progress {
        eprintln!("[count_lines] scanning & measuring...");
    }

    // Scan and measure files. If an error occurs and strict mode is
    // enabled, return the error. Otherwise, emit a warning and
    // continue with an empty result set.
    let mut stats = match compute::process_entries(&config) {
        Ok(v) => v,
        Err(e) => {
            if config.strict {
                return Err(e).context("failed to measure entries");
            }
            eprintln!("[warn] {}", e);
            Vec::new()
        }
    };

    // Apply sorting according to the user's specification.
    compute::apply_sort(&mut stats, &config);
    // Emit the results in the requested format.
    output::emit(&stats, &config).context("failed to emit output")?;
    Ok(())
}