#![allow(clippy::multiple_crate_versions)]

//! Entry point for the `count_lines` application.
//!
//! This module exposes the layered module structure (`app` →
//! `interface` → `domain` → `foundation`) and delegates execution to
//! the application layer.

pub mod app;
pub mod domain;
pub mod foundation;
pub mod interface;
pub mod version;

pub use version::VERSION;

fn main() -> anyhow::Result<()> {
    app::run()
}
