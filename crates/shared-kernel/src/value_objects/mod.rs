// crates/shared-kernel/src/value_objects/mod.rs
pub mod counts;
pub mod file_info;
pub mod file_meta;

pub use counts::{CharCount, LineCount, WordCount};
pub use file_info::{FileExtension, FileName, FilePath, FileSize, ModificationTime};
pub use file_meta::FileMeta;
