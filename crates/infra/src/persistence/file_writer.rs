use std::{fs::File, io::BufWriter, path::Path};

/// Helper utilities for writing files.
pub struct FileWriter;

impl FileWriter {
    /// Create a buffered writer targeting `path`.
    pub fn create(path: &Path) -> std::io::Result<BufWriter<File>> {
        File::create(path).map(BufWriter::new)
    }
}
