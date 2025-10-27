// src/types.rs
use chrono::{DateTime, Local};
use serde::Serialize;
use std::path::PathBuf;

/// Metadata associated with a file entry.
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub size: u64,
    pub mtime: Option<DateTime<Local>>, 
    pub is_text: bool,
    pub ext: String,
    pub name: String,
}

/// A path together with its metadata.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub meta: FileMeta,
}

/// Computed statistics for a file.
#[derive(Debug, Clone)]
pub struct FileStats {
    pub path: PathBuf,
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
    pub size: u64,
    pub mtime: Option<DateTime<Local>>, 
    pub ext: String,
    pub name: String,
}

impl FileStats {
    pub fn new(
        path: PathBuf,
        lines: usize,
        chars: usize,
        words: Option<usize>,
        meta: &FileMeta,
    ) -> Self {
        Self {
            path,
            lines,
            chars,
            words,
            size: meta.size,
            mtime: meta.mtime,
            ext: meta.ext.clone(),
            name: meta.name.clone(),
        }
    }
}

/// Summary statistics over all processed files.
#[derive(Debug, Clone)]
pub struct Summary {
    pub lines: usize,
    pub chars: usize,
    pub words: usize,
    pub files: usize,
}

impl Summary {
    pub fn from_stats(stats: &[FileStats]) -> Self {
        let (lines, chars, words) = stats.iter().fold((0, 0, 0), |(l, c, w), s| {
            (l + s.lines, c + s.chars, w + s.words.unwrap_or(0))
        });
        Self {
            lines,
            chars,
            words,
            files: stats.len(),
        }
    }
}

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