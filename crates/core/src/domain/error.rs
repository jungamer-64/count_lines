// crates/core/src/domain/error.rs
use glob::PatternError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Invalid filter expression: {0}")]
    InvalidFilterExpression(String),
    
    #[error("Invalid pattern '{pattern}': {source}")]
    InvalidPattern {
        pattern: String,
        #[source]
        source: PatternError,
    },
}

// infrastructureレイヤーのエラー
#[derive(Debug, Error)]
pub enum InfrastructureError {
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Failed to parse {format} output: {source}")]
    SerializationError {
        format: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    #[error("Git operation failed: {0}")]
    GitError(String),
    
    #[error("Thread pool creation failed: {0}")]
    ThreadPoolCreation(String),
}
