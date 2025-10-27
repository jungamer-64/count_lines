#![allow(clippy::multiple_crate_versions)]

//! Library crate for the `count_lines` application.
//!
//! Exposes the layered module structure (bootstrap → application → presentation
//! → domain/infrastructure/shared) along with convenience re-exports for callers.

pub mod application;
pub mod bootstrap;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;
pub mod version;

pub use application::{ConfigOptions, ConfigQueryService, FilterOptions};
pub use bootstrap::{run, run_with_config};
pub use domain::config::Config;
pub use presentation::cli::{self, Args};
pub use version::VERSION;

/// Execute the application by parsing CLI arguments from the process environment.
pub fn run_from_cli() -> anyhow::Result<()> {
    let config = presentation::cli::load_config()?;
    run_with_config(config)
}

/// Execute the application using pre-parsed CLI arguments.
pub fn run_from_args(args: Args) -> anyhow::Result<()> {
    let config = presentation::cli::build_config(args)?;
    run_with_config(config)
}
