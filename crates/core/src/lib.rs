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
pub mod shared;
pub mod version;

pub use application::{ConfigOptions, ConfigQueryService, FilterOptions};
pub use bootstrap::run_with_config;
pub use domain::config::Config;
pub use error::{CountLinesError, Result};
pub use version::VERSION;
