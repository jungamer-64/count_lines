// crates/shared-kernel/src/error.rs
use std::path::PathBuf;

use thiserror::Error;

/// Root error type shared across the workspace.
#[derive(Debug, Error)]
pub enum CountLinesError {
    /// Adds human context while preserving original error as the source.
    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<CountLinesError>,
    },

    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),

    #[error("Application error: {0}")]
    Application(#[from] ApplicationError),

    #[error("Presentation error: {0}")]
    Presentation(#[from] PresentationError),
}

pub type Result<T> = std::result::Result<T, CountLinesError>;

/// Domain-layer specific errors.
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration { reason: String },

    #[error("Invalid filter expression: {expression} - {details}")]
    InvalidFilterExpression { expression: String, details: String },

    #[error("Invalid pattern '{pattern}': {details}")]
    InvalidPattern {
        pattern: String,
        details: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Invalid sort specification: {spec}")]
    InvalidSortSpec { spec: String },

    #[error("Range validation failed: {field} must be between {min} and {max}")]
    RangeValidation {
        field: String,
        min: String,
        max: String,
    },
}

pub type DomainResult<T> = std::result::Result<T, DomainError>;

/// Application-layer errors.
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to collect file entries: {reason}")]
    FileCollectionFailed {
        reason: String,
        #[source]
        source: Option<Box<CountLinesError>>,
    },

    #[error("Failed to measure file statistics: {reason}")]
    MeasurementFailed {
        reason: String,
        #[source]
        source: Option<Box<CountLinesError>>,
    },

    #[error("Failed to present output: {reason}")]
    PresentationFailed {
        reason: String,
        #[source]
        source: Option<Box<CountLinesError>>,
    },

    #[error("Command execution failed: {command} - {reason}")]
    CommandFailed { command: String, reason: String },

    #[error("Query execution failed: {query} - {reason}")]
    QueryFailed { query: String, reason: String },
}

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

/// Infrastructure-layer errors.
#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write file '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse {format} output: {details}")]
    SerializationError { format: String, details: String },

    #[error("Git operation failed: {operation} - {details}")]
    GitError { operation: String, details: String },

    #[error("Thread pool creation failed: {details}")]
    ThreadPoolCreation { details: String },

    #[error("File system operation failed: {operation} on '{path}': {source}")]
    FileSystemOperation {
        operation: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Measurement error: failed to measure '{path}': {reason}")]
    MeasurementError { path: PathBuf, reason: String },

    #[error("Output error: {message}")]
    OutputError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

pub type InfraResult<T> = std::result::Result<T, InfrastructureError>;

/// Presentation-layer errors.
#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("CLI argument parsing failed: {argument} - {reason}")]
    ArgumentParsing { argument: String, reason: String },

    #[error("Invalid CLI value: {flag} = {value} - {reason}")]
    InvalidValue {
        flag: String,
        value: String,
        reason: String,
    },

    #[error("Configuration building failed: {0}")]
    ConfigBuildFailed(String),
}

pub type PresentationResult<T> = std::result::Result<T, PresentationError>;

impl From<std::io::Error> for InfrastructureError {
    fn from(err: std::io::Error) -> Self {
        Self::OutputError { message: err.to_string(), source: Some(Box::new(err)) }
    }
}

impl From<std::io::Error> for CountLinesError {
    fn from(err: std::io::Error) -> Self {
        InfrastructureError::from(err).into()
    }
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError {
            format: "JSON".to_string(),
            details: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for CountLinesError {
    fn from(err: serde_json::Error) -> Self {
        InfrastructureError::from(err).into()
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for InfrastructureError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::SerializationError {
            format: "YAML".to_string(),
            details: err.to_string(),
        }
    }
}

#[cfg(feature = "yaml")]
impl From<serde_yaml::Error> for CountLinesError {
    fn from(err: serde_yaml::Error) -> Self {
        InfrastructureError::from(err).into()
    }
}

#[cfg(feature = "eval")]
impl From<evalexpr::EvalexprError> for DomainError {
    fn from(err: evalexpr::EvalexprError) -> Self {
        Self::InvalidFilterExpression {
            expression: String::new(),
            details: err.to_string(),
        }
    }
}

/// Extension trait to add additional context to results.
pub trait ErrorContext<T> {
    fn context(self, context: impl Into<String>) -> Result<T>;
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<CountLinesError>,
{
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| CountLinesError::Context {
            context: context.into(),
            source: Box::new(e.into()),
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| CountLinesError::Context {
            context: f(),
            source: Box::new(e.into()),
        })
    }
}
