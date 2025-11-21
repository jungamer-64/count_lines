//! # Use Cases
//!
//! Application-level orchestration logic.
//!
//! This crate coordinates domain logic and infrastructure adapters
//! to implement specific use cases:
//!
//! - [`orchestrator`]: Main orchestration logic for counting files
//! - [`dto`]: Data transfer objects for use case boundaries
//!
//! Use cases depend on both domain and ports, but not on infrastructure.

#![allow(clippy::multiple_crate_versions)]

pub mod dto;
pub mod orchestrator;

pub use dto::CountEntriesOutput;
pub use orchestrator::CountPaths;
