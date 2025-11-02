// crates/shared-kernel/src/value_objects/file_meta.rs
use chrono::{DateTime, Local};

/// Minimal file metadata captured during enumeration.
#[derive(Debug, Clone)]
pub struct FileMeta {
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub is_text: bool,
    pub ext: String,
    pub name: String,
}
