#![allow(clippy::multiple_crate_versions)]

//! Library crate for the `count_lines` application.
//!
//! Exposes the layered module structure (`app` → `interface` → `domain`
//! → `foundation`) along with convenience re-exports for callers.

pub mod app;
pub mod domain;
pub mod foundation;
pub mod interface;
pub mod version;

pub use app::{run, run_with_config};
pub use domain::config::Config;
pub use interface::cli::{self, Args};
pub use version::VERSION;

/// Execute the application by parsing CLI arguments from the process environment.
pub fn run_from_cli() -> anyhow::Result<()> {
    let config = interface::cli::load_config()?;
    run_with_config(config)
}

/// Execute the application using pre-parsed CLI arguments.
pub fn run_from_args(args: Args) -> anyhow::Result<()> {
    let config = interface::cli::build_config(args)?;
    run_with_config(config)
}
