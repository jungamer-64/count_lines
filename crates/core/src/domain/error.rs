// ============================================================================
// crates/core/src/domain/error.rs (新規追加)
// ============================================================================
use std::fmt;

/// ドメイン固有のエラー型
#[derive(Debug)]
pub enum DomainError {
    InvalidConfiguration(String),
    InvalidFilterExpression(String),
    InvalidPattern(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {msg}"),
            Self::InvalidFilterExpression(msg) => write!(f, "Invalid filter expression: {msg}"),
            Self::InvalidPattern(msg) => write!(f, "Invalid pattern: {msg}"),
        }
    }
}

impl std::error::Error for DomainError {}