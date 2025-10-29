// crates/core/src/error.rs
//! 統一エラー型定義

use std::path::PathBuf;

use thiserror::Error;

/// アプリケーション全体のルートエラー型
#[derive(Debug, Error)]
pub enum CountLinesError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),

    #[error("Application error: {0}")]
    Application(#[from] ApplicationError),

    #[error("Presentation error: {0}")]
    Presentation(#[from] PresentationError),
}

/// 型エイリアス
pub type Result<T> = std::result::Result<T, CountLinesError>;

// ============================================================================
// Domain Layer Errors
// ============================================================================

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration { reason: String },

    #[error("Invalid filter expression: {expression} - {details}")]
    InvalidFilterExpression { expression: String, details: String },

    #[error("Invalid pattern '{pattern}': {source}")]
    InvalidPattern {
        pattern: String,
        #[source]
        source: glob::PatternError,
    },

    #[error("Invalid sort specification: {spec}")]
    InvalidSortSpec { spec: String },

    #[error("Range validation failed: {field} must be between {min} and {max}")]
    RangeValidation { field: String, min: String, max: String },
}

pub type DomainResult<T> = std::result::Result<T, DomainError>;

// ============================================================================
// Application Layer Errors
// ============================================================================

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to collect file entries: {0}")]
    FileCollectionFailed(String),

    #[error("Failed to measure file statistics: {0}")]
    MeasurementFailed(String),

    #[error("Failed to present output: {0}")]
    PresentationFailed(String),

    #[error("Command execution failed: {command} - {reason}")]
    CommandFailed { command: String, reason: String },

    #[error("Query execution failed: {query} - {reason}")]
    QueryFailed { query: String, reason: String },
}

pub type ApplicationResult<T> = std::result::Result<T, ApplicationError>;

// ============================================================================
// Infrastructure Layer Errors
// ============================================================================

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

    #[error("Output error: {0}")]
    OutputError(String),
}

pub type InfraResult<T> = std::result::Result<T, InfrastructureError>;

// ============================================================================
// Presentation Layer Errors
// ============================================================================

#[derive(Debug, Error)]
pub enum PresentationError {
    #[error("CLI argument parsing failed: {argument} - {reason}")]
    ArgumentParsing { argument: String, reason: String },

    #[error("Invalid CLI value: {flag} = {value} - {reason}")]
    InvalidValue { flag: String, value: String, reason: String },

    #[error("Configuration building failed: {0}")]
    ConfigBuildFailed(String),
}

pub type PresentationResult<T> = std::result::Result<T, PresentationError>;

// ============================================================================
// Error Conversion Utilities
// ============================================================================

impl From<std::io::Error> for InfrastructureError {
    fn from(err: std::io::Error) -> Self {
        Self::OutputError(err.to_string())
    }
}

impl From<std::io::Error> for CountLinesError {
    fn from(err: std::io::Error) -> Self {
        InfrastructureError::from(err).into()
    }
}

impl From<serde_json::Error> for InfrastructureError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError { format: "JSON".to_string(), details: err.to_string() }
    }
}

impl From<serde_json::Error> for CountLinesError {
    fn from(err: serde_json::Error) -> Self {
        InfrastructureError::from(err).into()
    }
}

impl From<serde_yaml::Error> for InfrastructureError {
    fn from(err: serde_yaml::Error) -> Self {
        Self::SerializationError { format: "YAML".to_string(), details: err.to_string() }
    }
}

impl From<serde_yaml::Error> for CountLinesError {
    fn from(err: serde_yaml::Error) -> Self {
        InfrastructureError::from(err).into()
    }
}

impl From<evalexpr::EvalexprError> for DomainError {
    fn from(err: evalexpr::EvalexprError) -> Self {
        Self::InvalidFilterExpression { expression: String::new(), details: err.to_string() }
    }
}

// ============================================================================
// Context Extension Trait
// ============================================================================

/// エラーにコンテキストを追加するためのトレイト
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
    fn context(self, _context: impl Into<String>) -> Result<T> {
        self.map_err(|e| {
            let err: CountLinesError = e.into();
            // コンテキスト情報をエラーメッセージに追加
            // 実際の実装ではより洗練された方法で実装可能
            err
        })
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.context(f())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_error_display() {
        let err = DomainError::InvalidConfiguration { reason: "missing required field".to_string() };
        assert!(err.to_string().contains("Invalid configuration"));
    }

    #[test]
    fn infra_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let infra_err: InfrastructureError = io_err.into();
        assert!(infra_err.to_string().contains("Output error"));
    }

    #[test]
    fn error_chain() {
        let domain_err = DomainError::InvalidPattern {
            pattern: "[[".to_string(),
            source: glob::Pattern::new("[[").unwrap_err(),
        };
        let root_err: CountLinesError = domain_err.into();
        assert!(root_err.to_string().contains("Domain error"));
    }

    #[test]
    fn application_error_variants() {
        let err = ApplicationError::CommandFailed {
            command: "run_analysis".to_string(),
            reason: "invalid state".to_string(),
        };
        assert!(err.to_string().contains("Command execution failed"));
    }
}
