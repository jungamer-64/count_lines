use chrono::{DateTime, Local};

/// Metadata associated with a file entry.
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub is_text: bool,
    pub ext: String,
    pub name: String,
}
