//! Re-export error types from the shared kernel crate for backwards compatibility.

pub use count_lines_shared_kernel::{
    ApplicationError, ApplicationResult, CountLinesError, DomainError, DomainResult, ErrorContext,
    InfraResult, InfrastructureError, PresentationError, PresentationResult, Result,
};
