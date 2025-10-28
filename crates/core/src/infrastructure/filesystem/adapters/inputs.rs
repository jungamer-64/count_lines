use std::{
    io::{BufRead, Read},
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::infrastructure::persistence::FileReader;

pub(crate) fn read_files_from_lines(path: &Path) -> Result<Vec<PathBuf>> {
    let reader = FileReader::open_buffered(path)?;
    Ok(reader
        .lines()
        .map_while(Result::ok)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect())
}

pub(crate) fn read_files_from_null(path: &Path) -> Result<Vec<PathBuf>> {
    let mut file = FileReader::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf
        .split(|&b| b == 0)
        .filter_map(|chunk| {
            if chunk.is_empty() {
                return None;
            }
            let s = String::from_utf8_lossy(chunk);
            let trimmed = s.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        })
        .collect())
}
