//! # Shared Kernel
//!
//! Common types and error handling shared across all crates.
//!
//! This crate provides the foundation for the entire application:
//!
//! - **Error Types**: Comprehensive error hierarchy for all layers

// crates/shared-kernel/src/lib.rs
#![allow(clippy::multiple_crate_versions)]

pub use error::{
    ApplicationError, ApplicationResult, CountLinesError, DomainError, DomainResult, ErrorContext,
    InfraResult, InfrastructureError, PresentationError, PresentationResult, Result,
};

pub mod error;
pub mod path;
pub mod value_objects;

pub use value_objects::{
    CharCount, FileExtension, FileMeta, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
};
