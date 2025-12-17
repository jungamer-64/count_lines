use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

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

    // Kept for generic analysis errors if needed, but intended to be phased out
    #[error("Analysis failed: {0}")]
    Analysis(String),

    #[error("Unknown extension: {0}")]
    UnknownExtension(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
