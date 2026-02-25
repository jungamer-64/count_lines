// crates/cli/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Engine(#[from] count_lines_engine::error::EngineError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Comparison error: {0}")]
    Comparison(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
