use serde::Deserialize;

/// A summary of lines, characters, words and file count from a JSON snapshot.
///
/// This mirrors the structure emitted by the JSON output format.
#[derive(Debug, Deserialize)]
pub(super) struct FileSummary {
    pub(super) lines: usize,
    pub(super) chars: usize,
    pub(super) words: Option<usize>,
    pub(super) files: usize,
}

/// Information about a single file from a JSON snapshot.
#[derive(Debug, Deserialize)]
pub(super) struct FileItem {
    pub file: String,
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
}

/// Top-level structure of a JSON snapshot.
#[derive(Debug, Deserialize)]
pub(super) struct Snapshot {
    pub(super) files: Vec<FileItem>,
    pub(super) summary: FileSummary,
}
