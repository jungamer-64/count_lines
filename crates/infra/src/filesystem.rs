// crates/infra/src/filesystem.rs
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

fn is_definitely_binary_ext(ext: &str) -> bool {
    matches!(
        ext,
        // archives/compressed
        "zip" | "7z" | "rar" | "gz" | "bz2" | "xz" | "zst" | "lz4" | "tar" |
        // executables / objects
        "exe" | "dll" | "so" | "dylib" | "o" | "a" | "wasm" | "bin" |
    // images
    "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "heic" | "avif" |
        // audio/video
        "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "mp4" | "mkv" | "mov" |
        // documents/binary formats
        "pdf" | "doc" | "docx" | "ppt" | "pptx" | "xls" | "xlsx" | "jar" | "apk" | "ipa" | "psd" | "iso" | "dmg" |
        // fonts / db / binary data
        "ttf" | "otf" | "woff" | "woff2" | "db" | "sqlite" | "sqlite3" | "parquet"
    )
}

fn detect_text(path: &Path, fast: bool, size: u64, ext: &str) -> bool {
    // Short-circuit common binary extensions to avoid unnecessary IO and
    // reduce false positives on quick sniffing large files.
    if is_definitely_binary_ext(ext) {
        return false;
    }

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
        // Platform-aware normalization for sorting and deduplication. On
        // Windows/NTFS we compare case-insensitively to avoid duplicates due
        // to case differences.
        // 事前計算されたキーで安定ソート（キー生成は各要素1回）
        #[cfg(windows)]
        fn norm_key(p: &Path) -> String {
            // lossy変換はここ1回。以後はcloneせず参照
            let s = p.to_string_lossy();
            s.to_lowercase()
        }
        #[cfg(unix)]
        fn norm_key(p: &Path) -> Vec<u8> {
            use std::os::unix::ffi::OsStrExt;
            p.as_os_str().as_bytes().to_vec()
        }
        #[cfg(all(not(windows), not(unix)))]
        fn norm_key(p: &Path) -> String {
            p.to_string_lossy().into_owned()
        }

        #[cfg(windows)]
        {
            // Build key once, sort by it, then dedup by adjacent equal keys.
            let mut keyed: Vec<(String, FileEntryDto)> =
                walk_entries.into_iter().map(|e| (norm_key(&e.path), e)).collect();
            keyed.sort_by(|a, b| a.0.cmp(&b.0));
            keyed.dedup_by(|a, b| a.0 == b.0);
            walk_entries = keyed.into_iter().map(|(_, e)| e).collect();
        }
        #[cfg(not(windows))]
        {
            // Use cached-key sort so we only create the normalization key once
            walk_entries.sort_by_cached_key(|e| norm_key(&e.path));
            walk_entries.dedup_by(|a, b| a.path == b.path);
        }
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
        // Avoid cloning PathBuf repeatedly: use a lightweight key.
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let mut seen: HashSet<Vec<u8>> = HashSet::new();
            paths.retain(|p| seen.insert(p.as_os_str().as_bytes().to_vec()));
        }
        #[cfg(not(unix))]
        {
            let mut seen: HashSet<String> = HashSet::new();
            paths.retain(|p| seen.insert(p.to_string_lossy().into_owned()));
        }
    }

    let mut entries = Vec::new();
    for path in paths {
        if !plan.include_hidden && is_hidden(&path) {
            continue;
        }
        if let Some(light) = build_meta_light(&path, plan.follow_links)
            && matcher.matches_file_light(&path, &light)
        {
            let is_text = detect_text(&path, plan.fast_text_detect, light.size, &light.ext);
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
            warn_msg(&format!("walk error: {}", err));
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
    build_entry_from_path_if_matches(path, matcher, include_hidden, fast_text, follow_links)
}

fn build_entry_from_path_if_matches(
    path: PathBuf,
    matcher: &PlanMatcher,
    include_hidden: bool,
    fast_text: bool,
    follow_links: bool,
) -> Option<FileEntryDto> {
    if !include_hidden && is_hidden(&path) {
        return None;
    }
    match build_meta_light(&path, follow_links) {
        Some(light) if matcher.matches_file_light(&path, &light) => {
            let is_text = detect_text(&path, fast_text, light.size, &light.ext);
            let meta = FileMetadata {
                size: light.size,
                mtime: light.mtime,
                is_text,
                ext: light.ext,
                name: light.name,
            };
            Some(to_port_entry(path, meta))
        }
        _ => None,
    }
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
    // Respect user's global gitignore and repository exclude files when possible
    // to reduce unnecessary IO during walks.
    builder.git_global(true);
    builder.git_exclude(true);
    builder.ignore(true);

    let matcher_for_dirs = Arc::clone(matcher);
    let include_hidden = plan.include_hidden;
    let no_default_prune = plan.no_default_prune;
    let follow_links = plan.follow_links;
    // Lightweight visited set to avoid following directory symlink loops when
    // follow_links is enabled. On Unix use (dev, ino) pairs which are cheap to
    // obtain. On non-Unix platforms fall back to canonicalized path tracking
    // but only for symlinked directories (to limit canonicalize calls).
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;

    #[cfg(unix)]
    let visited_dirs: Arc<Mutex<HashSet<(u64, u64)>>> = Arc::new(Mutex::new(HashSet::new()));
    #[cfg(unix)]
    let visited_dirs_cloned = Arc::clone(&visited_dirs);

    #[cfg(not(unix))]
    let visited_dirs: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
    #[cfg(not(unix))]
    let visited_dirs_cloned = Arc::clone(&visited_dirs);
    builder.filter_entry(move |entry| {
        if let Some(ft) = entry.file_type() {
            // Prepare to optionally fetch metadata once and reuse it to
            // avoid duplicate stat calls (helps on heavy walk workloads).
            let mut maybe_md: Option<std::fs::Metadata> = None;
            // Consider directory-like entries. If the entry reports as a
            // directory, it's clearly directory-like. If it's a symlink and
            // the plan requests following links, check the target's metadata
            // to see whether it is a directory and apply pruning accordingly.
            let is_dir_like = if ft.is_dir() {
                if follow_links {
                    maybe_md = std::fs::metadata(entry.path()).ok();
                }
                true
            } else if follow_links && entry.path_is_symlink() {
                // Stat once to determine if symlink target is a directory
                maybe_md = std::fs::metadata(entry.path()).ok();
                maybe_md.as_ref().map(|m| m.is_dir()).unwrap_or(false)
            } else {
                false
            };

            if is_dir_like {
                // Loop prevention: if following links, try to detect whether
                // we've already visited the same directory (by inode on Unix
                // or by canonical path on other platforms). If so, skip
                // descending into it.
                #[cfg(unix)]
                if follow_links {
                    if let Some(md) = maybe_md.as_ref() {
                        let key = (md.dev(), md.ino());
                        let mut set = visited_dirs_cloned.lock().unwrap();
                        if !set.insert(key) {
                            return false; // already visited
                        }
                    } else if let Ok(md) = std::fs::metadata(entry.path()) {
                        let key = (md.dev(), md.ino());
                        let mut set = visited_dirs_cloned.lock().unwrap();
                        if !set.insert(key) {
                            return false; // already visited
                        }
                    }
                }

                #[cfg(not(unix))]
                if follow_links && entry.path_is_symlink() {
                    if let Ok(canon) = std::fs::canonicalize(entry.path()) {
                        let mut set = visited_dirs_cloned.lock().unwrap();
                        if !set.insert(canon) {
                            return false; // already visited
                        }
                    }
                }

                return matcher_for_dirs.should_visit_dir(entry.path(), include_hidden, no_default_prune);
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
    let mut is_first = true;
    for line in reader.lines() {
        let mut line =
            line.map_err(|source| InfrastructureError::FileRead { path: path.to_path_buf(), source })?;
        if is_first {
            // Strip BOM (U+FEFF) only on the very first line if present.
            if let Some(s) = line.strip_prefix('\u{feff}') {
                line = s.to_owned();
            }
            is_first = false;
        }
        // Lines already split on '\n'. Remove a trailing '\r' if present
        // (CRLF handling) but preserve other whitespace.
        let s = line.trim_end_matches('\r');
        if !s.is_empty() {
            files.push(PathBuf::from(s));
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
            // Convert raw bytes to PathBuf preserving non-UTF-8 paths on Unix.
            Some(path_from_bytes(chunk))
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
            warn_msg(&format!("{} is not a git repo (git ls-files skipped)", root.display()));
            continue;
        }
        for chunk in output.stdout.split(|&b| b == 0) {
            if !chunk.is_empty() {
                files.push(root.join(path_from_bytes(chunk)));
            }
        }
    }
    #[cfg(windows)]
    {
        // Compute lowercase keys once per path, sort by key and remove
        // duplicates. This avoids calling `to_lowercase()` twice per item
        // (once during sort and once during retain).
        let mut keyed: Vec<(String, PathBuf)> = files
            .into_iter()
            .map(|p| (p.to_string_lossy().to_lowercase(), p))
            .collect();
        keyed.sort_by(|a, b| a.0.cmp(&b.0));
        keyed.dedup_by(|a, b| a.0 == b.0);
        files = keyed.into_iter().map(|(_k, p)| p).collect();
    }
    #[cfg(not(windows))]
    {
        files.sort();
        files.dedup();
    }
    Ok(files)
}

// Convert raw bytes to a PathBuf with platform-aware handling. This avoids
// lossy conversions for non-UTF-8 paths on Unix while providing a reasonable
// fallback on Windows (git on Windows typically emits UTF-8).
#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    PathBuf::from(std::ffi::OsStr::from_bytes(bytes))
}

#[cfg(windows)]
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    match std::str::from_utf8(bytes) {
        Ok(s) => PathBuf::from(s),
        Err(_) => PathBuf::from(String::from_utf8_lossy(bytes).into_owned()),
    }
}

#[derive(Debug)]
struct FileMetadata {
    size: u64,
    mtime: Option<DateTime<Local>>,
    is_text: bool,
    ext: String,
    name: String,
}

// Shared threshold for flushing per-thread collectors
const FLUSH_THRESHOLD: usize = 64;

// Per-thread buffer to batch appends into the global shared vector to
// reduce mutex contention during parallel walks.
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

// Lightweight warning helper so we can centralize stderr usage and later
// replace with a structured logger (tracing/log) if desired.
fn warn_msg(msg: &str) {
    eprintln!("[warn] {}", msg);
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
                        true
                    } else if s.starts_with(b"%PDF-")
                        || s.starts_with(b"PK\x03\x04")
                        || s.starts_with(b"PK\x05\x06")
                        || s.starts_with(b"MZ")
                        || s.starts_with(b"\x7FELF")
                        || s.starts_with(b"\x89PNG\r\n\x1A\n")
                        || s.starts_with(b"\xFF\xD8\xFF")
                        || s.starts_with(b"GIF87a")
                        || s.starts_with(b"GIF89a")
                        || s.starts_with(b"OggS")
                        || (s.starts_with(b"RIFF") && (s.get(8..12) == Some(b"WAVE") || s.get(8..12) == Some(b"WEBP")))
                        || s.get(4..8) == Some(b"ftyp")
                        || s.starts_with(b"\x1F\x8B")
                        || s.starts_with(b"BZh")
                        || s.starts_with(b"\xFD7zXZ\x00")
                        || s.starts_with(b"\x28\xB5\x2F\xFD")
                        || s.starts_with(b"\x04\x22\x4D\x18")
                        || s.starts_with(b"7z\xBC\xAF\x27\x1C")
                        || s.starts_with(b"Rar!\x1A\x07")
                        || s.starts_with(b"ID3")
                    {
                        false
                    } else {
                        !s.contains(&0)
                    }
                }
                Err(err) => {
                    warn_msg(&format!("quick_text_check read error for {}: {}", path.display(), err));
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
                        if memchr::memchr(0, &buf[..n]).is_some() {
                            return false;
                        }
                    }
                    Err(err) => {
                        warn_msg(&format!("strict_text_check read error for {}: {}", path.display(), err));
                        return false;
                    }
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
        // Normalize size and mtime ranges (defensive: swap when reversed)
        let (lo, hi) = normalize_size_range(plan.size_range);
        let (since, until) = normalize_mtime_range(plan.mtime_since, plan.mtime_until);
        // Normalize pattern strings on Windows (replace backslashes) so that
        // path-like detection (contains '/') behaves as expected.
        #[cfg(windows)]
        let include_pattern_strings = plan.include_patterns.iter().map(|p| p.replace('\\', "/")).collect::<Vec<_>>();
        #[cfg(not(windows))]
        let include_pattern_strings = plan.include_patterns.clone();
        #[cfg(windows)]
        let exclude_pattern_strings = plan.exclude_patterns.iter().map(|p| p.replace('\\', "/")).collect::<Vec<_>>();
        #[cfg(not(windows))]
        let exclude_pattern_strings = plan.exclude_patterns.clone();

        Ok(Self {
            include_patterns: compile_patterns(&plan.include_patterns)?,
            include_pattern_strings,
            exclude_patterns: compile_patterns(&plan.exclude_patterns)?,
            exclude_pattern_strings,
            include_paths: compile_patterns(&plan.include_paths)?,
            exclude_paths: compile_patterns(&plan.exclude_paths)?,
            exclude_dirs: compile_patterns(&plan.exclude_dirs)?,
            ext_filters: plan.ext_filters.iter().map(|ext| ext.to_lowercase()).collect(),
            size_min: lo,
            size_max: hi,
            mtime_since: since,
            mtime_until: until,
        })
    }

    fn should_visit_dir(&self, path: &Path, include_hidden: bool, no_default_prune: bool) -> bool {
        if !include_hidden && is_hidden(path) {
            return false;
        }

        if !no_default_prune && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // On Windows, prune matching should be case-insensitive to
            // account for NTFS case-insensitivity (e.g. "node_modules" vs "Node_Modules").
            #[cfg(windows)]
            let hit = DEFAULT_PRUNE_DIRS.iter().any(|d| d.eq_ignore_ascii_case(name));
            #[cfg(not(windows))]
            let hit = DEFAULT_PRUNE_DIRS.contains(&name);
            if hit {
                return false;
            }
        }

        !self.exclude_dirs.iter().any(|matcher| matcher.is_match(path))
    }

    fn matches_file(&self, path: &Path, meta: &FileMetadata) -> bool {
        // Use unified name-or-path matching so that heavy (full) matching
        // behaves the same way as the lightweight path-aware matchers.
        self.matches_name_or_path(path)
            && self.ext_allowed(&meta.ext)
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
    fn ext_allowed(&self, ext: &str) -> bool {
        self.ext_filters.is_empty() || self.ext_filters.contains(ext)
    }

    // Lightweight matching using only name/path/size/mtime/ext (avoids text checks)
    fn matches_file_light(&self, path: &Path, meta: &FileMetaLight) -> bool {
        self.matches_name_or_path(path)
            && self.ext_allowed(&meta.ext)
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

// Normalize size range (swap when reversed). Extracted for clarity and testability.
fn normalize_size_range(r: (Option<u64>, Option<u64>)) -> (Option<u64>, Option<u64>) {
    let (mut lo, mut hi) = r;
    if let (Some(a), Some(b)) = (lo, hi) && a > b {
        std::mem::swap(&mut lo, &mut hi);
    }
    (lo, hi)
}

// Normalize mtime range (swap when reversed). Extracted for clarity and testability.
fn normalize_mtime_range(
    since: Option<DateTime<Local>>,
    until: Option<DateTime<Local>>,
) -> (Option<DateTime<Local>>, Option<DateTime<Local>>) {
    match (since, until) {
        (Some(a), Some(b)) if a > b => (Some(b), Some(a)),
        other => other,
    }
}

fn compile_patterns(patterns: &[String]) -> Result<Vec<GlobMatcher>> {
    #[cfg(windows)]
    {
        let converted: Vec<String> = patterns.iter().map(|p| p.replace('\\', "/")).collect();
        converted
            .iter()
            .map(|pattern| {
                Glob::new(pattern)
                    .map_err(|err| InfrastructureError::OutputError(format!("invalid glob pattern '{pattern}': {err}")))
                    .map(|glob| glob.compile_matcher())
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
    #[cfg(not(windows))]
    {
        patterns
            .iter()
            .map(|pattern| {
                Glob::new(pattern)
                    .map_err(|err| InfrastructureError::OutputError(format!("invalid glob pattern '{pattern}': {err}")))
                    .map(|glob| glob.compile_matcher())
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::*;

    fn base_plan() -> FileEnumerationPlan {
        FileEnumerationPlan {
            roots: vec![],
            follow_links: false,
            include_hidden: true,
            no_default_prune: true,
            fast_text_detect: true,
            include_patterns: vec![],
            exclude_patterns: vec![],
            include_paths: vec![],
            exclude_paths: vec![],
            exclude_dirs: vec![],
            ext_filters: vec![],
            size_range: (None, None),
            mtime_since: None,
            mtime_until: None,
            files_from: None,
            files_from0: None,
            use_git: false,
        }
    }

    fn make_metadata(ext: &str, size: u64) -> FileMetadata {
        FileMetadata { size, mtime: None, is_text: true, ext: ext.to_string(), name: format!("file.{ext}") }
    }

    #[test]
    fn matcher_honours_include_patterns_on_paths() {
        let mut plan = base_plan();
        plan.include_patterns = vec!["src/**".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher");

        assert!(matcher.matches_name_or_path(Path::new("src/lib.rs")));
        assert!(!matcher.matches_name_or_path(Path::new("tests/lib.rs")));
    }

    #[test]
    fn matcher_swaps_reversed_mtime_range() {
        let mut plan = base_plan();
        let now = chrono::Local::now();
        let past = now - chrono::Duration::days(1);
        // intentionally reversed
        plan.mtime_since = Some(now);
        plan.mtime_until = Some(past);

        let matcher = PlanMatcher::new(&plan).expect("build matcher");
        let since = matcher.mtime_since.expect("since");
        let until = matcher.mtime_until.expect("until");
        assert!(since <= until, "expected mtime_since <= mtime_until after normalization");
    }

    #[test]
    fn matcher_filters_by_extension_for_full_metadata() {
        let mut plan = base_plan();
        plan.ext_filters = vec!["rs".into(), "md".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher");

        assert!(matcher.matches_file(Path::new("src/file.rs"), &make_metadata("rs", 10)));
        assert!(matcher.matches_file(Path::new("guide.md"), &make_metadata("md", 10)));
        assert!(!matcher.matches_file(Path::new("notes.txt"), &make_metadata("txt", 10)));
    }

    #[test]
    fn matcher_applies_size_range_to_light_metadata() {
        let mut plan = base_plan();
        plan.size_range = (Some(10), Some(100));
        let matcher = PlanMatcher::new(&plan).expect("build matcher");

        let meta_small = FileMetaLight { size: 5, mtime: None, ext: "rs".into(), name: "a.rs".into() };
        let meta_ok = FileMetaLight { size: 50, mtime: None, ext: "rs".into(), name: "b.rs".into() };
        let meta_large = FileMetaLight { size: 500, mtime: None, ext: "rs".into(), name: "c.rs".into() };

        assert!(!matcher.matches_size_light(&meta_small));
        assert!(matcher.matches_size_light(&meta_ok));
        assert!(!matcher.matches_size_light(&meta_large));
    }

    #[test]
    fn materialise_paths_deduplicates_and_preserves_order() {
        let dir = tempdir().expect("temp dir");
        let file_a = dir.path().join("a.txt");
        let file_b = dir.path().join("b.txt");
        std::fs::write(&file_a, "hello").expect("write a");
        std::fs::write(&file_b, "world").expect("write b");

        let plan = base_plan();
        let matcher = PlanMatcher::new(&plan).expect("build matcher");
        let paths = vec![file_a.clone(), file_b.clone(), file_a.clone()];

        let entries = materialise_paths(paths, &plan, &matcher).expect("materialise succeeds");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].path, file_a);
        assert_eq!(entries[1].path, file_b);
    }

    #[test]
    fn filter_respects_include_hidden() {
        let mut plan = base_plan();
        plan.include_hidden = false;
        let m = PlanMatcher::new(&plan).unwrap();
        // .git is considered hidden (dot-prefixed)
        assert!(!m.should_visit_dir(Path::new(".git"), /*include_hidden=*/false, /*no_default_prune=*/true));
    }

    #[test]
    fn include_and_exclude_paths_priority() {
        let mut plan = base_plan();
        plan.include_paths = vec!["src/**".into()];
        plan.exclude_paths = vec!["src/gen/**".into()];
        let m = PlanMatcher::new(&plan).unwrap();
        assert!(m.matches_name_or_path(Path::new("src/lib.rs")));
        assert!(!m.matches_name_or_path(Path::new("src/gen/out.rs")));
    }

    #[cfg(unix)]
    #[test]
    fn build_meta_light_follow_links_switch() {
        use std::os::unix::fs::symlink;
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("t.txt");
        std::fs::write(&target, "x").unwrap();
        let link = dir.path().join("l");
        symlink(&target, &link).unwrap();

        assert!(build_meta_light(&link, true).is_some());
        assert!(build_meta_light(&link, false).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_loop_is_prevented() {
        use std::os::unix::fs::symlink;
        use std::sync::Arc;

        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("root");
        std::fs::create_dir(&root).unwrap();
        // create a symlink inside root that points to root itself
        symlink(&root, root.join("loop")).unwrap();

        let mut plan = base_plan();
        plan.roots = vec![root.clone()];
        plan.follow_links = true;
        plan.include_hidden = true;
        plan.no_default_prune = true;

        // Should not hang; return value can be empty since there are no files.
        let matcher = PlanMatcher::new(&plan).unwrap();
        let arc = Arc::new(matcher);
        let entries = super::collect_entries_from_root(&root, &plan, &arc).unwrap();
        assert!(entries.is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn compile_patterns_normalizes_backslashes() {
        let mut plan = base_plan();
        plan.include_patterns = vec![r"src\**\*.rs".into()];
        let m = PlanMatcher::new(&plan).unwrap();
        // These paths should match the include pattern after normalization
        assert!(m.matches_name_or_path(Path::new("src\\lib.rs")));
        assert!(m.matches_name_or_path(Path::new("src/main.rs")));
    }

    #[cfg(windows)]
    #[test]
    fn windows_dedup_ignores_case_after_sort() {
        use std::path::PathBuf;
        let mut v = vec![
            FileEntryDto { path: PathBuf::from("SRC\\A.TXT"), is_text: false, size: 0, ext: "txt".into(), name: "A.TXT".into(), mtime: None },
            FileEntryDto { path: PathBuf::from("src\\a.txt"), is_text: false, size: 0, ext: "txt".into(), name: "a.txt".into(), mtime: None },
        ];
        v.sort_by_cached_key(|e| e.path.to_string_lossy().to_lowercase());
        v.dedup_by(|a, b| a.path.to_string_lossy().to_lowercase() == b.path.to_string_lossy().to_lowercase());
        assert_eq!(v.len(), 1);
    }

    #[test]
    fn detect_text_short_circuits_binary_ext() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("x.pdf");
        std::fs::write(&p, b"%PDF-1.4\nhello").unwrap();
        // extension-based short-circuit should detect as non-text without I/O
        assert!(!detect_text(&p, /*fast*/ true, 100, "pdf"));
    }

    #[test]
    fn mtime_filter_rejects_when_meta_missing() {
        let mut plan = base_plan();
        let now = chrono::Local::now();
        plan.mtime_since = Some(now - chrono::Duration::hours(1));
        let m = PlanMatcher::new(&plan).unwrap();

        let meta = FileMetaLight { size: 1, mtime: None, ext: "txt".into(), name: "a.txt".into() };
        assert!(!m.matches_mtime_light(&meta));
    }

    #[test]
    fn very_large_files_use_quick_check() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("big.txt");
        std::fs::write(&p, "x").unwrap();
        // Passing an explicit size larger than the sniff threshold should force quick_text_check
        assert!(detect_text(&p, /*fast*/ false, LARGE_TEXT_SNIFF_THRESHOLD + 1, "txt"));
    }

    #[test]
    fn normalize_size_range_swaps() {
        assert_eq!(normalize_size_range((Some(20), Some(10))), (Some(10), Some(20)));
    }
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

#[cfg(target_os = "macos")]
fn is_hidden(path: &Path) -> bool {
    use std::os::macos::fs::MetadataExt;
    // Check the UF_HIDDEN flag on macOS (Finder hidden files) first.
    if let Ok(md) = std::fs::symlink_metadata(path) {
        const UF_HIDDEN: u32 = 0x0000_8000;
        if (md.st_flags() as u32) & UF_HIDDEN != 0 {
            return true;
        }
    }
    path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with('.')).unwrap_or(false)
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
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
    ".vscode",
    ".terraform",
    "bazel-bin",
    "bazel-out",
];
