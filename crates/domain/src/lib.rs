//! # Domain
//!
//! Core domain models and business logic for the count_lines application.
//!
//! This crate contains pure domain logic with no external dependencies:
//!
//! - [`config`]: Configuration domain model
//! - [`model`]: Core entities like `FileEntry`, `FileStats`, `Summary`
//! - [`analytics`]: Aggregation and sorting logic
//! - [`grouping`]: Grouping strategies (by extension, directory, mtime)
//! - [`options`]: Output format, sort keys, and other option types
//! - [`value_objects`]: Domain-specific value objects

#![allow(clippy::multiple_crate_versions)]

pub mod analytics;
pub mod config;
pub mod grouping;
pub mod model;
pub mod options;
pub mod value_objects;
