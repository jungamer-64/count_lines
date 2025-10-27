// src/files.rs
use chrono::{DateTime, Local};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{Config, Filters};
use crate::types::{FileEntry, FileMeta};

/// Default directories pruned unless `no_default_prune` is set.
const DEFAULT_PRUNE_DIRS: &[&str] = &[
    ".git", ".hg", ".svn", "node_modules", ".venv", "venv", "build", "dist", "target",
    ".cache", ".direnv", ".mypy_cache", ".pytest_cache", "coverage", "__pycache__",
    ".idea", ".next", ".nuxt",
];

/// Collect the list of file entries to be processed based on the provided configuration.
pub fn collect_entries(config: &Config) -> anyhow::Result<Vec<FileEntry>> {
    if let Some(ref from0) = config.files_from0 {
        return read_files_from_null(from0).map(|files| to_entries(files, config));
    }
    if let Some(ref from) = config.files_from {
        return read_files_from_lines(from).map(|files| to_entries(files, config));
    }
    if config.use_git {
        if let Ok(files) = collect_git_files(config) {
            return Ok(to_entries(files, config));
        }
    }
    collect_walk_entries(config)
}

fn to_entries(files: Vec<PathBuf>, config: &Config) -> Vec<FileEntry> {
    files
        .into_iter()
        .filter_map(|p| {
            FileMeta::from_path(&p, config)
                .map(|meta| FileEntry { path: p, meta })
        })
        .collect()
}

fn read_files_from_lines(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .filter_map(Result::ok)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect())
}

fn read_files_from_null(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut file = File::open(path)?;
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

fn collect_git_files(config: &Config) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in &config.paths {
        let output = std::process::Command::new("git")
            .args([
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
                "--",
                ".",
            ])
            .current_dir(root)
            .output()?;
        if !output.status.success() {
            anyhow::bail!("git ls-files failed");
        }
        for chunk in output.stdout.split(|&b| b == 0) {
            if let Some(path_str) = parse_git_output_chunk(chunk) {
                files.push(root.join(path_str));
            }
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn parse_git_output_chunk(chunk: &[u8]) -> Option<String> {
    if chunk.is_empty() {
        return None;
    }
    let s = String::from_utf8_lossy(chunk).trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .map_or(false, |name| name.to_string_lossy().starts_with('.'))
}

fn should_process_entry(entry: &walkdir::DirEntry, config: &Config) -> bool {
    let path = entry.path();
    if !config.hidden && is_hidden(path) {
        return false;
    }
    if !config.no_default_prune && entry.file_type().is_dir() {
        let name = entry.file_name().to_string_lossy();
        if DEFAULT_PRUNE_DIRS.contains(&name.as_ref()) {
            return false;
        }
    }
    if entry.file_type().is_dir() {
        return !config
            .filters
            .exclude_dirs
            .iter()
            .any(|p| p.matches_path(path));
    }
    true
}

/// Walk the filesystem and collect entries matching the configured filters.
pub fn collect_walk_entries(config: &Config) -> anyhow::Result<Vec<FileEntry>> {
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
            if let Some(meta) = FileMeta::from_path(path, config) {
                entries.push(FileEntry {
                    path: path.to_path_buf(),
                    meta,
                });
            }
        }
    }
    Ok(entries)
}

struct PathMatcher;

impl PathMatcher {
    fn matches(path: &Path, config: &Config) -> bool {
        let filters = &config.filters;
        Self::matches_name(path, filters)
            && Self::matches_path_patterns(path, filters)
            && Self::matches_extension(path, filters)
            && Self::matches_metadata(path, config)
    }
    fn matches_name(path: &Path, filters: &Filters) -> bool {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        if !filters.include_patterns.is_empty()
            && !filters
                .include_patterns
                .iter()
                .any(|p| p.matches(&name))
        {
            return false;
        }
        !filters.exclude_patterns.iter().any(|p| p.matches(&name))
    }
    fn matches_path_patterns(path: &Path, filters: &Filters) -> bool {
        if !filters.include_paths.is_empty()
            && !filters
                .include_paths
                .iter()
                .any(|p| p.matches_path(path))
        {
            return false;
        }
        !filters.exclude_paths.iter().any(|p| p.matches_path(path))
    }
    fn matches_extension(path: &Path, filters: &Filters) -> bool {
        if filters.ext_filters.is_empty() {
            return true;
        }
        path.extension()
            .and_then(|e| Some(e.to_string_lossy().to_lowercase()))
            .map_or(false, |ext| filters.ext_filters.contains(&ext))
    }
    fn matches_metadata(path: &Path, config: &Config) -> bool {
        let filters = &config.filters;
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => return true,
        };
        if !filters.size_range.contains(metadata.len()) {
            return false;
        }
        Self::matches_mtime(&metadata, config)
    }
    fn matches_mtime(metadata: &std::fs::Metadata, config: &Config) -> bool {
        let Ok(modified_sys) = metadata.modified() else { return true };
        let modified: DateTime<Local> = modified_sys.into();
        if let Some(since) = config.mtime_since {
            if modified < since {
                return false;
            }
        }
        if let Some(until) = config.mtime_until {
            if modified > until {
                return false;
            }
        }
        true
    }
}

impl FileMeta {
    /// Construct file metadata from a path according to configuration.
    pub fn from_path(path: &Path, config: &Config) -> Option<Self> {
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
        Some(Self {
            size,
            mtime,
            is_text,
            ext,
            name,
        })
    }
    fn quick_text_check(path: &Path) -> bool {
        let Ok(mut file) = File::open(path) else { return false };
        let mut buf = [0u8; 1024];
        let n = file.read(&mut buf).unwrap_or(0);
        !buf[..n].contains(&0)
    }
    fn strict_text_check(path: &Path) -> bool {
        let Ok(mut file) = File::open(path) else { return false };
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).is_ok() && !buf.contains(&0)
    }
}