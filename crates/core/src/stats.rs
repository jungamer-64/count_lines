use serde::{Deserialize, Serialize};

/// Pure analysis result, independent of file system metadata.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Number of lines.
    pub lines: usize,
    /// Number of characters.
    pub chars: usize,
    /// Number of words (if counted).
    pub words: Option<usize>,
    /// Source Lines of Code (if counted).
    pub sloc: Option<usize>,
    /// Whether the content was detected as binary.
    pub is_binary: bool,
}

impl AnalysisResult {
    /// Creates a new default `AnalysisResult`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
