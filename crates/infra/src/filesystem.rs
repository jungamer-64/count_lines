use std::{
    collections::HashSet,
    io::{BufRead, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Local};
use count_lines_ports::filesystem::{FileEntryDto, FileEnumerationPlan, FileEnumerator};
use count_lines_shared_kernel::{InfrastructureError, Result};
use globset::{Glob, GlobMatcher};
use ignore::WalkBuilder;

use crate::persistence::FileReader;

const LARGE_TEXT_SNIFF_THRESHOLD: u64 = 16 * 1024 * 1024; // 16 MiB

#[derive(Debug)]
struct FileMetaLight {
    size: u64,
    mtime: Option<DateTime<Local>>,
    ext: String,
    name: String,
}

// Accept follow_links to make the behaviour consistent with WalkBuilder's
// follow_links flag. When follow_links is false we use symlink_metadata and
// explicitly exclude symlinks from being treated as regular files.
fn build_meta_light(path: &Path, follow_links: bool) -> Option<FileMetaLight> {
    let metadata =
        if follow_links { std::fs::metadata(path).ok()? } else { std::fs::symlink_metadata(path).ok()? };

    // If the caller requested not to follow links, treat symlinks as not-a-file.
    if !follow_links && metadata.file_type().is_symlink() {
        return None;
    }

    // Only consider regular files here. Directories and other special files
    // should not be treated as file entries.
    if !metadata.is_file() {
        return None;
    }

    let size = metadata.len();
    let mtime = metadata.modified().ok().map(DateTime::<Local>::from);
    let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    Some(FileMetaLight { size, mtime, ext, name })
}

fn detect_text(path: &Path, fast: bool, size: u64) -> bool {
    // For very large files prefer quick sniff to avoid O(file_size) reads.
    if size >= LARGE_TEXT_SNIFF_THRESHOLD {
        return quick_text_check(path);
    }
    if fast { quick_text_check(path) } else { strict_text_check(path) }
}

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

    let entries = if let Some(paths) = initial_paths(plan)? {
        // When the user supplies an explicit file list, preserve the order they
        // provided (materialise_paths will honor that). Do not sort.
        materialise_paths(paths, plan, &matcher)?
    } else {
        // For walk-based collection (multiple roots, git lists, etc.) ensure a
        // deterministic order and remove duplicates from overlapping roots.
        let mut walk_entries = walk_roots(plan, matcher)?;
        walk_entries.sort_by(|a, b| a.path.cmp(&b.path));
        walk_entries.dedup_by(|a, b| a.path == b.path);
        walk_entries
    };

    Ok(entries)
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
    mut paths: Vec<PathBuf>,
    plan: &FileEnumerationPlan,
    matcher: &PlanMatcher,
) -> Result<Vec<FileEntryDto>> {
    // Order-preserving dedup: prefer the first occurrence and drop later
    // duplicates. This keeps user-supplied ordering stable while removing
    // redundant work.
    {
        let mut seen = HashSet::new();
        paths.retain(|p| seen.insert(p.clone()));
    }

    let mut entries = Vec::new();
    for path in paths {
        if !plan.include_hidden && is_hidden(&path) {
            continue;
        }
        if let Some(light) = build_meta_light(&path, plan.follow_links) {
            if matcher.matches_file_light(&path, &light) {
                let is_text = detect_text(&path, plan.fast_text_detect, light.size);
                let meta = FileMetadata {
                    size: light.size,
                    mtime: light.mtime,
                    is_text,
                    ext: light.ext,
                    name: light.name,
                };
                entries.push(to_port_entry(path, meta));
            }
        }
    }
    Ok(entries)
}

// Small helper: process a single `ignore` walk result. Pulled out of the closure
// to lower the cyclomatic complexity of the parallel walk closure.
// Note: helper `handle_walk_result` removed in favor of a lighter-weight
// per-thread buffering strategy implemented in `collect_entries_from_root`.

// Attempt to construct a `FileEntryDto` from a walk result. Returns `None`
// for non-files, errors, or entries that don't match the given matcher.
fn try_build_entry_from_result(
    result: std::result::Result<ignore::DirEntry, ignore::Error>,
    matcher: &PlanMatcher,
    include_hidden: bool,
    fast_text: bool,
    follow_links: bool,
) -> Option<FileEntryDto> {
    let entry = match result {
        Ok(e) => e,
        Err(err) => {
            eprintln!("[warn] walk error: {err}");
            return None;
        }
    };

    // Treat as a candidate if it's a regular file, or if it's a symlink and
    // the plan requested following links. The `ignore` crate reports the
    // DirEntry's file_type without following symlinks, so we must allow
    // symlinks here and defer to `build_meta_light` (which will follow or
    // not follow based on `follow_links`) to decide final inclusion.
    let is_candidate = match entry.file_type() {
        Some(ft) => ft.is_file() || (follow_links && ft.is_symlink()),
        None => false,
    };
    if !is_candidate {
        return None;
    }

    let path = entry.into_path();
    if !include_hidden && is_hidden(&path) {
        return None;
    }

    match build_meta_light(&path, follow_links) {
        Some(light) if matcher.matches_file_light(&path, &light) => {
            let is_text = detect_text(&path, fast_text, light.size);
            let meta = FileMetadata {
                size: light.size,
                mtime: light.mtime,
                is_text,
                ext: light.ext,
                name: light.name,
            };
            return Some(to_port_entry(path, meta));
        }
        _ => {}
    }
    None
}

// Collect entries for a single root path. This extracts the per-root builder
// and parallel run logic out of `walk_roots` so the top-level function stays small.
fn collect_entries_from_root(
    root: &Path,
    plan: &FileEnumerationPlan,
    matcher: &Arc<PlanMatcher>,
) -> Result<Vec<FileEntryDto>> {
    let mut builder = WalkBuilder::new(root);
    builder.follow_links(plan.follow_links);
    // Let the walker suppress hidden files when the plan requests hiding
    // them. This avoids emitting hidden paths to our closure and reduces IO.
    builder.hidden(!plan.include_hidden);
    builder.git_ignore(true);

    let dir_matcher = Arc::clone(matcher);
    let include_hidden = plan.include_hidden;
    let no_default_prune = plan.no_default_prune;
    let follow_links = plan.follow_links;
    builder.filter_entry(move |entry| {
        if let Some(ft) = entry.file_type() {
            // Consider directory-like entries. If the entry reports as a
            // directory, it's clearly directory-like. If it's a symlink and
            // the plan requests following links, check the target's metadata
            // to see whether it is a directory and apply pruning accordingly.
            let is_dir_like = if ft.is_dir() {
                true
            } else if follow_links && ft.is_symlink() {
                std::fs::metadata(entry.path()).map(|m| m.is_dir()).unwrap_or(false)
            } else {
                false
            };

            if is_dir_like {
                return dir_matcher.should_visit_dir(entry.path(), include_hidden, no_default_prune);
            }
        }
        true
    });

    // parallel walk: aggregate entries into a thread-safe Vec
    let entries_ref: Arc<Mutex<Vec<FileEntryDto>>> = Arc::new(Mutex::new(Vec::new()));
    let entries_clone = Arc::clone(&entries_ref);
    let matcher_ref = Arc::clone(matcher);
    let include_hidden = plan.include_hidden;
    let fast_text = plan.fast_text_detect;
    let follow_links = plan.follow_links;

    // Use a thread-local buffer to reduce mutex contention: collect entries in a
    // local Vec and flush them to the shared vector in batches. The
    // LocalCollector ensures leftover items are flushed when the thread exits.
    const FLUSH_THRESHOLD: usize = 64;
    struct LocalCollector {
        buf: Vec<FileEntryDto>,
        shared: Arc<Mutex<Vec<FileEntryDto>>>,
    }
    impl LocalCollector {
        fn with_capacity(shared: Arc<Mutex<Vec<FileEntryDto>>>) -> Self {
            Self { buf: Vec::with_capacity(FLUSH_THRESHOLD), shared }
        }
    }
    impl Drop for LocalCollector {
        fn drop(&mut self) {
            if !self.buf.is_empty() {
                let mut guard = self.shared.lock().unwrap();
                guard.append(&mut self.buf);
            }
        }
    }

    builder.build_parallel().run(move || {
        let matcher = Arc::clone(&matcher_ref);
        let shared = Arc::clone(&entries_clone);
        let mut collector = LocalCollector::with_capacity(shared);
        Box::new(move |result| {
            if let Some(dto) =
                try_build_entry_from_result(result, &matcher, include_hidden, fast_text, follow_links)
            {
                collector.buf.push(dto);
                if collector.buf.len() >= FLUSH_THRESHOLD {
                    let mut guard = collector.shared.lock().unwrap();
                    guard.append(&mut collector.buf);
                }
            }
            ignore::WalkState::Continue
        })
    });

    // merge local entries into main vector
    let mut guard = entries_ref.lock().unwrap();
    let mut collected = Vec::new();
    collected.append(&mut guard);
    Ok(collected)
}

fn walk_roots(plan: &FileEnumerationPlan, matcher: PlanMatcher) -> Result<Vec<FileEntryDto>> {
    let matcher = Arc::new(matcher);
    let mut entries = Vec::new();

    for root in &plan.roots {
        let mut root_entries = collect_entries_from_root(root, plan, &matcher)?;
        entries.append(&mut root_entries);
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
            // Skip roots that are not git repositories rather than erroring the whole operation.
            eprintln!("[warn] {} is not a git repo (git ls-files skipped)", root.display());
            continue;
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

#[allow(dead_code)]
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
    match FileReader::open(path) {
        Ok(mut file) => {
            // Read a larger sample to allow BOM detection for UTF-16/32.
            let mut buf = [0u8; 4096];
            match file.read(&mut buf) {
                Ok(n) => {
                    let s = &buf[..n];
                    // Allow common BOMs: UTF-8, UTF-16 (LE/BE), UTF-32 (LE/BE).
                    if s.starts_with(&[0xEF, 0xBB, 0xBF])
                        || s.starts_with(&[0xFF, 0xFE])
                        || s.starts_with(&[0xFE, 0xFF])
                        || s.starts_with(&[0xFF, 0xFE, 0x00, 0x00])
                        || s.starts_with(&[0x00, 0x00, 0xFE, 0xFF])
                    {
                        return true;
                    }
                    !s.contains(&0)
                }
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
    match FileReader::open(path) {
        Ok(mut file) => {
            // Stream the file in fixed-size chunks to avoid allocating for
            // very large files while searching for NUL bytes.
            let mut buf = [0u8; 64 * 1024];
            loop {
                match file.read(&mut buf) {
                    Ok(0) => return true, // EOF reached, no NUL seen
                    Ok(n) => {
                        if buf[..n].contains(&0) {
                            return false;
                        }
                    }
                    Err(_) => return false,
                }
            }
        }
        Err(_) => false,
    }
}

#[allow(dead_code)]
struct PlanMatcher {
    include_patterns: Vec<GlobMatcher>,
    include_pattern_strings: Vec<String>,
    exclude_patterns: Vec<GlobMatcher>,
    exclude_pattern_strings: Vec<String>,
    include_paths: Vec<GlobMatcher>,
    exclude_paths: Vec<GlobMatcher>,
    exclude_dirs: Vec<GlobMatcher>,
    ext_filters: HashSet<String>,
    size_min: Option<u64>,
    size_max: Option<u64>,
    mtime_since: Option<DateTime<Local>>,
    mtime_until: Option<DateTime<Local>>,
}

#[allow(dead_code)]
impl PlanMatcher {
    fn new(plan: &FileEnumerationPlan) -> Result<Self> {
        Ok(Self {
            include_patterns: compile_patterns(&plan.include_patterns)?,
            include_pattern_strings: plan.include_patterns.clone(),
            exclude_patterns: compile_patterns(&plan.exclude_patterns)?,
            exclude_pattern_strings: plan.exclude_patterns.clone(),
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
            match path.file_name().and_then(|n| n.to_str()) {
                Some(name) if DEFAULT_PRUNE_DIRS.contains(&name) => return false,
                _ => (),
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

    // Lightweight matching using only name/path/size/mtime/ext (avoids text checks)
    fn matches_file_light(&self, path: &Path, meta: &FileMetaLight) -> bool {
        self.matches_name_or_path(path)
            && self.matches_extension_light(meta)
            && self.matches_size_light(meta)
            && self.matches_mtime_light(meta)
    }

    fn matches_name_or_path(&self, path: &Path) -> bool {
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // include patterns: if present, at least one must match
        if !self.include_patterns.is_empty() && !self.include_patterns_match(path, fname) {
            return false;
        }

        // exclude patterns: none must match
        if self.exclude_patterns_match(path, fname) {
            return false;
        }

        // explicit include/exclude path matchers
        if !self.include_paths.is_empty() && !self.include_paths.iter().any(|m| m.is_match(path)) {
            return false;
        }
        if self.exclude_paths.iter().any(|m| m.is_match(path)) {
            return false;
        }

        true
    }

    fn include_patterns_match(&self, path: &Path, fname: &str) -> bool {
        if self.include_patterns.is_empty() {
            return true;
        }
        for (pat, matcher) in self.include_pattern_strings.iter().zip(self.include_patterns.iter()) {
            let looks_like_path = pat.contains('/') || pat.contains("**");
            if looks_like_path {
                if matcher.is_match(path) {
                    return true;
                }
            } else if matcher.is_match(fname) {
                return true;
            }
        }
        false
    }

    fn exclude_patterns_match(&self, path: &Path, fname: &str) -> bool {
        for (pat, matcher) in self.exclude_pattern_strings.iter().zip(self.exclude_patterns.iter()) {
            let looks_like_path = pat.contains('/') || pat.contains("**");
            if looks_like_path {
                if matcher.is_match(path) {
                    return true;
                }
            } else if matcher.is_match(fname) {
                return true;
            }
        }
        false
    }

    fn matches_extension_light(&self, meta: &FileMetaLight) -> bool {
        if self.ext_filters.is_empty() {
            return true;
        }
        self.ext_filters.contains(&meta.ext)
    }

    fn matches_size_light(&self, meta: &FileMetaLight) -> bool {
        if self.size_min.is_some_and(|min| meta.size < min) {
            return false;
        }
        if self.size_max.is_some_and(|max| meta.size > max) {
            return false;
        }
        true
    }

    fn matches_mtime_light(&self, meta: &FileMetaLight) -> bool {
        let need_filter = self.mtime_since.is_some() || self.mtime_until.is_some();
        match (need_filter, meta.mtime) {
            (false, _) => true,
            (true, Some(m)) => {
                if self.mtime_since.is_some_and(|since| m < since) {
                    return false;
                }
                if self.mtime_until.is_some_and(|until| m > until) {
                    return false;
                }
                true
            }
            (true, None) => false,
        }
    }

    fn matches_size(&self, meta: &FileMetadata) -> bool {
        if self.size_min.is_some_and(|min| meta.size < min) {
            return false;
        }
        if self.size_max.is_some_and(|max| meta.size > max) {
            return false;
        }
        true
    }

    fn matches_mtime(&self, meta: &FileMetadata) -> bool {
        let need_filter = self.mtime_since.is_some() || self.mtime_until.is_some();
        match (need_filter, meta.mtime) {
            (false, _) => true,
            (true, Some(m)) => {
                if self.mtime_since.is_some_and(|since| m < since) {
                    return false;
                }
                if self.mtime_until.is_some_and(|until| m > until) {
                    return false;
                }
                true
            }
            (true, None) => false,
        }
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

#[cfg(target_os = "windows")]
fn is_hidden(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    // Use symlink_metadata so that when `path` is itself a symlink we inspect
    // the symlink entry's attributes rather than following the link and
    // inspecting the target's attributes.
    if let Ok(md) = std::fs::symlink_metadata(path) {
        if (md.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0 {
            return true;
        }
    }
    path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with('.')).unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
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
