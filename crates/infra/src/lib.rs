//! # Infrastructure
//!
//! Reusable infrastructure implementations for the count_lines application.
//!
//! This crate provides concrete implementations of external concerns:
//!
//! - [`filesystem`]: File enumeration, Git integration, directory traversal
//! - [`measurement`]: File metrics calculation (lines, chars, words)
//! - [`cache`]: Incremental cache storage and retrieval
//! - [`persistence`]: File read/write helpers
//! - [`watch`]: File system watching service
//! - [`platform`]: Platform-specific utilities
//!
//! These implementations can be reused across different applications.

// crates/infra/src/lib.rs
#![allow(clippy::multiple_crate_versions)]

pub mod cache;
pub mod filesystem;
pub mod measurement;
pub mod persistence;
pub mod platform;
pub mod watch;
