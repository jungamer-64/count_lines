// src/version.rs
//! Defines the version string for the application.
//!
//! This constant uses the `CARGO_PKG_VERSION` environment variable to
//! automatically stay in sync with the version specified in `Cargo.toml`.

/// Application version derived from Cargo.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
