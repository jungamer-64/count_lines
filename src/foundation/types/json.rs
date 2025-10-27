use serde::Serialize;

/// Top-level JSON output structure for structured formats.
#[derive(Debug, Serialize)]
pub struct JsonOutput {
    pub version: &'static str,
    pub files: Vec<JsonFile>,
    pub summary: JsonSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub by: Option<Vec<JsonGroup>>,
}

/// JSON representation of a single file.
#[derive(Debug, Serialize)]
pub struct JsonFile {
    pub file: String,
    pub lines: usize,
    pub chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<usize>,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtime: Option<String>,
    pub ext: String,
}

/// JSON representation of summary information.
#[derive(Debug, Serialize)]
pub struct JsonSummary {
    pub lines: usize,
    pub chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub words: Option<usize>,
    pub files: usize,
}

/// A row within a grouped aggregation.
#[derive(Debug, Serialize)]
pub struct JsonGroupRow {
    pub key: String,
    pub lines: usize,
    pub chars: usize,
    pub count: usize,
}

/// A group of aggregated rows.
#[derive(Debug, Serialize)]
pub struct JsonGroup {
    pub label: String,
    pub rows: Vec<JsonGroupRow>,
}
