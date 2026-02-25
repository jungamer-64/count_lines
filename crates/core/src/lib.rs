// crates/core/src/lib.rs
#![no_std]
#![warn(missing_docs, missing_debug_implementations)]

//! # count_lines_core
//!
//! Core counting logic for `count_lines` tool, designed to be `no_std` compatible.
//!
//! ## Features
//!
//! - **Line Counting**: High-performance line counting.
//! - **SLOC Counting**: Source Lines of Code (excluding comments/blanks) for 150+ languages.
//! - **Word/Char Counting**: Unicode-aware character and word counting.
//!
//! ## Usage
//!
//! ```rust
//! use count_lines_core::counter::count_bytes;
//! use count_lines_core::config::AnalysisConfig;
//!
//! let content = b"fn main() {\n    println!(\"Hello\");\n}\n";
//! let stats = count_bytes(content, "rs", &AnalysisConfig::default());
//! assert_eq!(stats.lines, 3);
//! if let Some(sloc) = stats.sloc {
//!     assert_eq!(sloc, 3);
//! }
//! ```
//!
//! ## Architecture
//!
//! - [`counter`]: Main entry point (`count_bytes`).
//! - [`language`]: Language-specific SLOC processors.
//! - [`stats`]: Statistical data structures.
//! - [`config`]: Configuration options.

#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]
extern crate alloc;

/// Configuration options for analysis.
pub mod config;
/// Main counting entry point.
pub mod counter;
/// Language-specific SLOC processors.
pub mod language;
/// Statistical result types.
pub mod stats;
