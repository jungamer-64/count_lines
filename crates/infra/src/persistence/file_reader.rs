// crates/infra/src/persistence/file_reader.rs
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

/// Convenience helpers for reading files with consistent error handling.
pub struct FileReader;

impl FileReader {
    /// Open the file at `path`.
    pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
        File::open(path.as_ref())
    }

    /// Open the file at `path` with buffered reading.
    pub fn open_buffered<P: AsRef<Path>>(path: P) -> std::io::Result<BufReader<File>> {
        Self::open(path).map(BufReader::new)
    }

    /// Read the entire file into memory.
    pub fn read_to_end<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        let mut file = Self::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Read a prefix of the file into `buf`, returning the byte count.
    pub fn read_prefix<P: AsRef<Path>>(path: P, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut file = Self::open(path)?;
        file.read(buf)
    }
}
