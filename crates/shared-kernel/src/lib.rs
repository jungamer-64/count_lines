// crates/shared-kernel/src/lib.rs
#![allow(clippy::multiple_crate_versions)]

pub use error::{
    ApplicationError, ApplicationResult, CountLinesError, DomainError, DomainResult, ErrorContext,
    InfraResult, InfrastructureError, PresentationError, PresentationResult, Result,
};

pub mod error;
pub mod value_objects;

pub use value_objects::{
    CharCount, FileExtension, FileMeta, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
};
