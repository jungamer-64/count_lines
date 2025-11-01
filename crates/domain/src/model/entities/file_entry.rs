use std::path::PathBuf;

use crate::value_objects::FileMeta;

/// A path together with its metadata.
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub meta: FileMeta,
}
