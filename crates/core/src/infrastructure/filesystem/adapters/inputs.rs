use std::{
    io::{BufRead, Read},
    path::{Path, PathBuf},
};

use crate::{
    error::{InfrastructureError, Result},
    infrastructure::persistence::FileReader,
};

pub(crate) fn read_files_from_lines(path: &Path) -> Result<Vec<PathBuf>> {
    let reader = FileReader::open_buffered(path)
        .map_err(|source| InfrastructureError::FileRead { path: path.to_path_buf(), source })?;

    let mut files = Vec::new();
    for line in reader.lines() {
        let line =
            line.map_err(|source| InfrastructureError::FileRead { path: path.to_path_buf(), source })?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            files.push(PathBuf::from(trimmed));
        }
    }

    Ok(files)
}

pub(crate) fn read_files_from_null(path: &Path) -> Result<Vec<PathBuf>> {
    let mut file = FileReader::open(path)
        .map_err(|source| InfrastructureError::FileRead { path: path.to_path_buf(), source })?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|source| InfrastructureError::FileRead { path: path.to_path_buf(), source })?;
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
