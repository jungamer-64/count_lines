use crate::domain::config::Config;
use crate::domain::model::FileMeta;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Domain service responsible for translating filesystem data into metadata value objects.
pub struct FileMetadataLoader;

impl FileMetadataLoader {
    pub fn build(path: &Path, config: &Config) -> Option<FileMeta> {
        let metadata = std::fs::metadata(path).ok()?;
        let size = metadata.len();
        let mtime = metadata.modified().ok().map(Into::into);

        let is_text = if config.fast_text_detect {
            Self::quick_text_check(path)
        } else {
            Self::strict_text_check(path)
        };
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Some(FileMeta {
            size,
            mtime,
            is_text,
            ext,
            name,
        })
    }

    fn quick_text_check(path: &Path) -> bool {
        let Ok(mut file) = File::open(path) else {
            return false;
        };
        let mut buf = [0u8; 1024];
        let n = file.read(&mut buf).unwrap_or_else(|e| {
            eprintln!(
                "[warn] quick_text_check read error for {}: {}",
                path.display(),
                e
            );
            0
        });
        !buf[..n].contains(&0)
    }

    fn strict_text_check(path: &Path) -> bool {
        let Ok(mut file) = File::open(path) else {
            return false;
        };
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).is_ok() && !buf.contains(&0)
    }
}
