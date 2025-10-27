use crate::domain::model::value_objects::FileMeta;
use std::path::PathBuf;

/// A path together with its metadata.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub meta: FileMeta,
}
