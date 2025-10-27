use crate::domain::config::Config;
use crate::domain::model::FileEntry;
use crate::infrastructure::filesystem::adapters::{
    PathMatcher, collect_git_files, read_files_from_lines, read_files_from_null,
    should_process_entry,
};
use crate::infrastructure::filesystem::services::metadata_loader::FileMetadataLoader;
use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

/// Application service responsible for discovering domain file entries.
pub struct FileEntryCollector;

impl FileEntryCollector {
    pub fn collect(config: &Config) -> Result<Vec<FileEntry>> {
        if let Some(ref from0) = config.files_from0 {
            let files = read_files_from_null(from0)?;
            return Ok(Self::materialise_entries(files, config));
        }
        if let Some(ref from) = config.files_from {
            let files = read_files_from_lines(from)?;
            return Ok(Self::materialise_entries(files, config));
        }
        if config.use_git {
            if let Ok(files) = collect_git_files(config) {
                return Ok(Self::materialise_entries(files, config));
            }
        }
        Self::collect_walk(config)
    }

    fn materialise_entries(files: Vec<PathBuf>, config: &Config) -> Vec<FileEntry> {
        files
            .into_iter()
            .filter_map(|path| {
                FileMetadataLoader::build(&path, config).map(|meta| FileEntry { path, meta })
            })
            .collect()
    }

    pub fn collect_walk(config: &Config) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        for root in &config.paths {
            let walker = WalkDir::new(root)
                .follow_links(config.follow)
                .into_iter()
                .filter_entry(|e| should_process_entry(e, config));

            for entry in walker.flatten() {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if !PathMatcher::matches(path, config) {
                    continue;
                }
                if let Some(meta) = FileMetadataLoader::build(path, config) {
                    entries.push(FileEntry {
                        path: path.to_path_buf(),
                        meta,
                    });
                }
            }
        }
        Ok(entries)
    }
}

pub fn collect_entries(config: &Config) -> Result<Vec<FileEntry>> {
    FileEntryCollector::collect(config)
}

pub fn collect_walk_entries(config: &Config) -> Result<Vec<FileEntry>> {
    FileEntryCollector::collect_walk(config)
}
