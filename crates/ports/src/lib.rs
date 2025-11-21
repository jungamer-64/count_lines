//! # Ports
//!
//! Interface definitions for external dependencies.
//!
//! This crate defines traits that abstract external concerns:
//!
//! - [`filesystem`]: File system access and directory traversal
//! - [`hashing`]: File content hashing for caching
//! - [`progress`]: Progress reporting for long-running operations
//!
//! These ports allow the domain and application layers to remain
//! independent of specific implementations.

// crates/ports/src/lib.rs
#![allow(clippy::multiple_crate_versions)]

pub mod filesystem;
pub mod hashing;
pub mod progress;
