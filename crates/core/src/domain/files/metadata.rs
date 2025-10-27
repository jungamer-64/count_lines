use crate::domain::config::Config;
use crate::foundation::types::FileMeta;
use std::fs::File;
use std::io::Read;
use std::path::Path;

impl FileMeta {
    /// Construct file metadata from a path according to configuration.
    pub fn from_path(path: &Path, config: &Config) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok()?;
        let size = metadata.len();
        let mtime = metadata.modified().ok().map(Into::into);

        let is_text = if config.fast_text_detect {
            quick_text_check(path)
        } else {
            strict_text_check(path)
        };
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Some(Self {
            size,
            mtime,
            is_text,
            ext,
            name,
        })
    }
}

fn quick_text_check(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut buf = [0u8; 1024];
    let n = file.read(&mut buf).unwrap_or(0);
    !buf[..n].contains(&0)
}

fn strict_text_check(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).is_ok() && !buf.contains(&0)
}
