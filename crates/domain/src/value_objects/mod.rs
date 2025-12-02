//! Value object modules for file statistics.

pub mod counts;
pub mod file_info;

pub use count_lines_shared_kernel::value_objects::FileMeta;
pub use counts::{CharCount, LineCount, SlocCount, WordCount};
pub use file_info::{FileExtension, FileName, FilePath, FileSize, ModificationTime};
