// crates/infra/src/persistence/file_writer.rs
use std::{fs, fs::File, io::{BufWriter, Write}, path::Path};

/// Helper utilities for writing files.
pub struct FileWriter;

impl FileWriter {
    /// Create a buffered writer targeting `path`.
    pub fn create<P: AsRef<Path>>(path: P) -> std::io::Result<BufWriter<File>> {
        File::create(path.as_ref()).map(BufWriter::new)
    }

    /// Atomically write `data` to `path` via a temp file and rename.
    /// Best-effort fsync is attempted where available to reduce corruption on crash.
    pub fn atomic_write<P: AsRef<Path>>(path: P, data: &[u8]) -> std::io::Result<()> {
        let path = path.as_ref();
        let parent = path.parent().ok_or_else(|| std::io::Error::other("path has no parent"))?;

        // Create a unique temp file name in the same directory to allow atomic rename.
        // Use PID + current time nanos to avoid an allocation loop while keeping
        // the name creation inexpensive.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let tmp = parent.join(format!(".{}.{}.tmp", std::process::id(), nanos));

        let file = File::create(&tmp)?;
        let mut w = BufWriter::new(file);
        w.write_all(data)?;
        w.flush()?;
        let _ = w.get_ref().sync_all();

        fs::rename(&tmp, path)?;

        // Attempt to sync parent directory to make the rename durable on Unix.
        #[cfg(unix)]
        {
            if let Ok(dir) = File::open(parent) {
                let _ = dir.sync_all();
            }
        }

        Ok(())
    }
}
