#![allow(clippy::multiple_crate_versions)]

//! Library crate for the `count_lines` application.
//!
//! Exposes the layered module structure (bootstrap → application → presentation
//! → domain/infrastructure/shared) along with convenience re-exports for callers.

pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod error;
pub mod infrastructure;
pub mod presentation;
pub mod shared;
pub mod version;

pub use application::{ConfigOptions, ConfigQueryService, FilterOptions};
pub use bootstrap::{run, run_with_config};
pub use domain::config::Config;
pub use error::{CountLinesError, Result};
pub use presentation::cli::{self, Args};
pub use version::VERSION;

/// Execute the application by parsing CLI arguments from the process environment.
///
/// # Errors
///
/// Returns an error when argument parsing or application startup fails.
pub fn run_from_cli() -> anyhow::Result<()> {
    let config = presentation::cli::load_config().map_err(anyhow::Error::from)?;
    run_with_config(config)
}

/// Execute the application using pre-parsed CLI arguments.
///
/// # Errors
///
/// Returns an error when the provided arguments are invalid or when the
/// application fails to start.
pub fn run_from_args(args: Args) -> anyhow::Result<()> {
    let config = presentation::cli::build_config(&args).map_err(anyhow::Error::from)?;
    run_with_config(config)
}
