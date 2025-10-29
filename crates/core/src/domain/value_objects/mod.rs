//! Value object modules for file statistics.

pub mod counts;
pub mod file_info;

pub use counts::{CharCount, LineCount, WordCount};
pub use file_info::{FileExtension, FileName, FilePath, FileSize, ModificationTime};
