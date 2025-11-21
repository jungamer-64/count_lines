#![allow(clippy::multiple_crate_versions)]

pub use count_lines_core::*;

pub mod cli;

pub use cli::{Args, build_config, load_config};

/// Execute the application by parsing CLI arguments from the process environment.
///
/// # Errors
///
/// Returns an error when argument parsing or application startup fails.
pub fn run_from_cli() -> anyhow::Result<()> {
    let config = cli::load_config().map_err(anyhow::Error::from)?;
    Ok(count_lines_core::run_with_config(config)?)
}

/// Execute the application using pre-parsed CLI arguments.
///
/// # Errors
///
/// Returns an error when the provided arguments are invalid or when the
/// application fails to start.
pub fn run_from_args(args: Args) -> anyhow::Result<()> {
    let config = cli::build_config(&args).map_err(anyhow::Error::from)?;
    Ok(count_lines_core::run_with_config(config)?)
}
