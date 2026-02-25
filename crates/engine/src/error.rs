use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Failed to read file '{path}': {source}")]
    FileRead {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Walk error: {0}")]
    Walk(#[from] ignore::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Watch error: {0}")]
    Watch(#[from] notify::Error),

    #[error("File size {size} is smaller than minimum {min}")]
    FileTooSmall { size: u64, min: u64 },

    #[error("File size {size} is larger than maximum {max}")]
    FileTooLarge { size: u64, max: u64 },

    #[error("File modified time {modified} is older than {since}")]
    FileTooOld { modified: String, since: String },

    #[error("Extension '{0}' is not allowed")]
    ExtensionNotAllowed(String),

    #[error("No extension found")]
    NoExtension,

    #[error("Invalid configuration: {0}")]
    Config(String),

    #[error("Invalid extension mapping: {0}")]
    InvalidExtMapping(String),

    #[error("Text processing failed: {0}")]
    TextProcessing(String),

    #[error("Cache operation failed: {0}")]
    Cache(String),

    #[error("Unknown extension: {0}")]
    UnknownExtension(String),

    #[error("IO error: {0}")]
    Io(std::io::Error),
}

pub type Result<T> = std::result::Result<T, EngineError>;
