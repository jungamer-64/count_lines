use serde::{Deserialize, Serialize};

/// Pure analysis result, independent of file system metadata.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
    pub sloc: Option<usize>,
    pub is_binary: bool,
}

impl AnalysisResult {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
