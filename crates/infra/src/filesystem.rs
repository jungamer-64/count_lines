use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};
use count_lines_ports::filesystem::{FileEntryDto, FileEnumerationPlan, FileEnumerator};
use count_lines_shared_kernel::{InfrastructureError, Result};
use globset::{Glob, GlobMatcher};
use ignore::WalkBuilder;

use crate::persistence::FileReader;

/// Filesystem adapter implementing the `FileEnumerator` port based on the enumeration plan.
#[derive(Debug, Default)]
pub struct PlanFileEnumerator;

impl PlanFileEnumerator {
    pub fn new() -> Self {
        Self
    }

    pub fn enumerate(plan: &FileEnumerationPlan) -> Result<Vec<FileEntryDto>> {
        enumerate_plan(plan)
    }
}

impl FileEnumerator for PlanFileEnumerator {
    fn collect(&self, plan: &FileEnumerationPlan) -> Result<Vec<FileEntryDto>> {
        Self::enumerate(plan)
    }
}

fn enumerate_plan(plan: &FileEnumerationPlan) -> Result<Vec<FileEntryDto>> {
    let matcher = PlanMatcher::new(plan)?;

    if let Some(paths) = initial_paths(plan)? {
        materialise_paths(paths, plan, &matcher)
    } else {
        walk_roots(plan, matcher)
    }
}

fn initial_paths(plan: &FileEnumerationPlan) -> Result<Option<Vec<PathBuf>>> {
    if let Some(path) = &plan.files_from0 {
        return read_files_from_null(path).map(Some);
    }
    if let Some(path) = &plan.files_from {
        return read_files_from_lines(path).map(Some);
    }
    if plan.use_git {
        return collect_git_files(&plan.roots).map(Some);
    }
    Ok(None)
}

fn materialise_paths(
    paths: Vec<PathBuf>,
    plan: &FileEnumerationPlan,
    matcher: &PlanMatcher,
) -> Result<Vec<FileEntryDto>> {
    let mut entries = Vec::new();
    for path in paths {
        if !plan.include_hidden && is_hidden(&path) {
            continue;
        }
        if let Some(meta) = build_metadata(&path, plan.fast_text_detect) {
            if matcher.matches_file(&path, &meta) {
                entries.push(to_port_entry(path, meta));
            }
        }
    }
    Ok(entries)
}

fn walk_roots(plan: &FileEnumerationPlan, matcher: PlanMatcher) -> Result<Vec<FileEntryDto>> {
    let matcher = std::sync::Arc::new(matcher);
    let mut entries = Vec::new();

    for root in &plan.roots {
        let mut builder = WalkBuilder::new(root);
        builder.follow_links(plan.follow_links);
        builder.hidden(false);
        builder.git_ignore(true);

        let dir_matcher = std::sync::Arc::clone(&matcher);
        let include_hidden = plan.include_hidden;
        let no_default_prune = plan.no_default_prune;
        builder.filter_entry(move |entry| {
            if let Some(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    return dir_matcher.should_visit_dir(entry.path(), include_hidden, no_default_prune);
                }
            }
            true
        });

        for result in builder.build() {
            let entry = match result {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("[warn] walk error: {err}");
                    continue;
                }
            };

            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.into_path();
            if !plan.include_hidden && is_hidden(&path) {
                continue;
            }

            if let Some(meta) = build_metadata(&path, plan.fast_text_detect) {
                if matcher.matches_file(&path, &meta) {
                    entries.push(to_port_entry(path, meta));
                }
            }
        }
    }

    Ok(entries)
}

fn to_port_entry(path: PathBuf, meta: FileMetadata) -> FileEntryDto {
    FileEntryDto {
        path,
        is_text: meta.is_text,
        size: meta.size,
        ext: meta.ext,
        name: meta.name,
        mtime: meta.mtime,
    }
}

fn read_files_from_lines(path: &Path) -> Result<Vec<PathBuf>> {
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

fn read_files_from_null(path: &Path) -> Result<Vec<PathBuf>> {
    let mut file = File::open(path)
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

fn collect_git_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in roots {
        let output = std::process::Command::new("git")
            .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "."])
            .current_dir(root)
            .output()
            .map_err(|source| InfrastructureError::FileSystemOperation {
                operation: "git ls-files".to_string(),
                path: root.clone(),
                source,
            })?;
        if !output.status.success() {
            let details = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(
                InfrastructureError::GitError { operation: "git ls-files".to_string(), details }.into()
            );
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

#[derive(Debug)]
struct FileMetadata {
    size: u64,
    mtime: Option<DateTime<Local>>,
    is_text: bool,
    ext: String,
    name: String,
}

fn build_metadata(path: &Path, fast_text_detect: bool) -> Option<FileMetadata> {
    let metadata = std::fs::metadata(path).ok()?;
    let size = metadata.len();
    let mtime = metadata.modified().ok().map(DateTime::<Local>::from);
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let is_text = if fast_text_detect { quick_text_check(path) } else { strict_text_check(path) };
    Some(FileMetadata { size, mtime, is_text, ext, name })
}

fn quick_text_check(path: &Path) -> bool {
    match File::open(path) {
        Ok(mut file) => {
            let mut buf = [0u8; 1024];
            match file.read(&mut buf) {
                Ok(n) => !buf[..n].contains(&0),
                Err(err) => {
                    eprintln!("[warn] quick_text_check read error for {}: {err}", path.display());
                    false
                }
            }
        }
        Err(_) => false,
    }
}

fn strict_text_check(path: &Path) -> bool {
    match File::open(path) {
        Ok(mut file) => {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).is_ok() && !buf.contains(&0)
        }
        Err(_) => false,
    }
}

struct PlanMatcher {
    include_patterns: Vec<GlobMatcher>,
    exclude_patterns: Vec<GlobMatcher>,
    include_paths: Vec<GlobMatcher>,
    exclude_paths: Vec<GlobMatcher>,
    exclude_dirs: Vec<GlobMatcher>,
    ext_filters: HashSet<String>,
    size_min: Option<u64>,
    size_max: Option<u64>,
    mtime_since: Option<DateTime<Local>>,
    mtime_until: Option<DateTime<Local>>,
}

impl PlanMatcher {
    fn new(plan: &FileEnumerationPlan) -> Result<Self> {
        Ok(Self {
            include_patterns: compile_patterns(&plan.include_patterns)?,
            exclude_patterns: compile_patterns(&plan.exclude_patterns)?,
            include_paths: compile_patterns(&plan.include_paths)?,
            exclude_paths: compile_patterns(&plan.exclude_paths)?,
            exclude_dirs: compile_patterns(&plan.exclude_dirs)?,
            ext_filters: plan.ext_filters.iter().map(|ext| ext.to_lowercase()).collect(),
            size_min: plan.size_range.0,
            size_max: plan.size_range.1,
            mtime_since: plan.mtime_since,
            mtime_until: plan.mtime_until,
        })
    }

    fn should_visit_dir(&self, path: &Path, include_hidden: bool, no_default_prune: bool) -> bool {
        if !include_hidden && is_hidden(path) {
            return false;
        }

        if !no_default_prune {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if DEFAULT_PRUNE_DIRS.contains(&name) {
                    return false;
                }
            }
        }

        !self.exclude_dirs.iter().any(|matcher| matcher.is_match(path))
    }

    fn matches_file(&self, path: &Path, meta: &FileMetadata) -> bool {
        self.matches_name(path)
            && self.matches_path(path)
            && self.matches_extension(meta)
            && self.matches_size(meta)
            && self.matches_mtime(meta)
    }

    fn matches_name(&self, path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };

        if !self.include_patterns.is_empty() && !self.include_patterns.iter().any(|m| m.is_match(name)) {
            return false;
        }

        !self.exclude_patterns.iter().any(|m| m.is_match(name))
    }

    fn matches_path(&self, path: &Path) -> bool {
        if !self.include_paths.is_empty() && !self.include_paths.iter().any(|m| m.is_match(path)) {
            return false;
        }

        !self.exclude_paths.iter().any(|m| m.is_match(path))
    }

    fn matches_extension(&self, meta: &FileMetadata) -> bool {
        if self.ext_filters.is_empty() {
            return true;
        }
        self.ext_filters.contains(&meta.ext)
    }

    fn matches_size(&self, meta: &FileMetadata) -> bool {
        if let Some(min) = self.size_min {
            if meta.size < min {
                return false;
            }
        }
        if let Some(max) = self.size_max {
            if meta.size > max {
                return false;
            }
        }
        true
    }

    fn matches_mtime(&self, meta: &FileMetadata) -> bool {
        if let Some(mtime) = meta.mtime {
            if let Some(since) = self.mtime_since {
                if mtime < since {
                    return false;
                }
            }
            if let Some(until) = self.mtime_until {
                if mtime > until {
                    return false;
                }
            }
        }
        true
    }
}

fn compile_patterns(patterns: &[String]) -> Result<Vec<GlobMatcher>> {
    patterns
        .iter()
        .map(|pattern| {
            Glob::new(pattern)
                .map_err(|err| {
                    InfrastructureError::OutputError(format!("invalid glob pattern '{pattern}': {err}"))
                })
                .map(|glob| glob.compile_matcher())
        })
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

fn is_hidden(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with('.')).unwrap_or(false)
}

const DEFAULT_PRUNE_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".venv",
    "venv",
    "build",
    "dist",
    "target",
    ".cache",
    ".direnv",
    ".mypy_cache",
    ".pytest_cache",
    "coverage",
    "__pycache__",
    ".idea",
    ".next",
    ".nuxt",
];
