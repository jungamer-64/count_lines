// crates/infra/src/filesystem.rs
use std::{
    collections::HashSet,
    io::{BufRead, Read},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Local, Utc};
use count_lines_ports::filesystem::{FileEntryDto, FileEnumerationPlan, FileEnumerator};
use count_lines_shared_kernel::{InfrastructureError, Result};
#[cfg(windows)]
use globset::GlobBuilder;
use globset::GlobMatcher;
use ignore::{WalkBuilder, overrides::OverrideBuilder};

use crate::persistence::FileReader;

const LARGE_TEXT_SNIFF_THRESHOLD: u64 = 16 * 1024 * 1024; // 16 MiB

#[derive(Debug)]
struct FileMetaLight {
    size: u64,
    mtime: Option<DateTime<Utc>>,
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
    let mtime = metadata.modified().ok().map(DateTime::<Utc>::from);
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
        "exe" | "dll" | "so" | "dylib" | "o" | "a" | "wasm" | "bin" | "msi" | "deb" | "rpm" | "rlib" | "rmeta" | "lib" |
    // images
    "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "tif" | "tiff" | "heic" | "avif" |
        // audio/video
        "mp3" | "flac" | "wav" | "m4a" | "aac" | "ogg" | "opus" | "mp4" | "m4v" | "mkv" | "mov" | "webm" |
        "avi" | "flv" | "mpg" | "mpeg" | "ts" | "m2ts" | "mts" |
        "aif" | "aiff" | "aifc" | "caf" | "mid" | "midi" |
        // documents/binary formats
        "pdf" | "doc" | "docx" | "ppt" | "pptx" | "xls" | "xlsx" | "jar" | "apk" | "ipa" | "psd" | "psb" | "iso"
            | "dmg" | "cab" |
        // fonts / db / binary data
        "ttf" | "otf" | "woff" | "woff2" | "db" | "sqlite" | "sqlite3" | "sqlite-wal" | "sqlite-shm" | "parquet"
            | "glb" | "arrow" | "feather" | "orc" |
        // data/ML formats
        "npz" | "npy" | "h5" | "hdf5" | "mat" | "rds" | "rdata" |
        "onnx" | "tflite" | "pb" | "pt" |
        // VM / disk images / databases
        "qcow2" | "vmdk" | "vdi" | "img" | "db3" | "mdb" | "accdb" |
        // mac-specific binaries
        "ds_store" | "icns" |
        // compressed variants that are effectively binary
        "svgz" | "lzma" | "z" |
        // other common binaries
        "class" | "pyc" | "pyo" | "crx"
    )
}

fn is_definitely_text_ext(ext: &str) -> bool {
    matches!(
        ext,
        "txt"
            | "text"
            | "log"
            | "csv"
            | "tsv"
            | "json"
            | "jsonl"
            | "ndjson"
            | "yaml"
            | "yml"
            | "toml"
            | "ini"
            | "cfg"
            | "conf"
            | "properties"
            | "xml"
            | "html"
            | "htm"
            | "md"
            | "rst"
            | "adoc"
            | "org"
            | "psv"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "rs"
            | "go"
            | "py"
            | "rb"
            | "java"
            | "kt"
            | "kts"
            | "swift"
            | "c"
            | "h"
            | "hpp"
            | "hh"
            | "cxx"
            | "cpp"
            | "cc"
            | "m"
            | "mm"
            | "scala"
            | "clj"
            | "cljs"
            | "cljc"
            | "edn"
            | "ex"
            | "exs"
            | "php"
            | "phtml"
            | "sql"
            | "psql"
            | "ps1"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "bat"
            | "cmd"
            | "gradle"
            | "groovy"
            | "dart"
            | "hs"
            | "erl"
            | "lua"
            | "nim"
            | "nix"
            | "zig"
            | "proto"
            | "thrift"
            | "cmake"
            | "make"
            | "mk"
            | "dockerfile"
            | "podspec"
            | "srt"
            | "vtt"
            | "hcl"
            | "tf"
            | "rtf"
            | "ipynb"
            | "mdx"
            | "vue"
            | "svelte"
            | "jinja"
            | "j2"
            | "jinja2"
            | "prisma"
            | "tex"
            | "bib"
            | "r"
            | "jl"
    )
}

fn is_definitely_text_name(name: &str) -> bool {
    matches!(
        name,
        "Dockerfile"
            | "Makefile"
            | "CMakeLists.txt"
            | ".gitignore"
            | ".gitattributes"
            | ".editorconfig"
            | "Pipfile"
            | "requirements.txt"
            | "Gemfile"
            | "Cargo.lock"
            | "Justfile"
            | "Cargo.toml"
            | "go.mod"
            | "go.sum"
            | "WORKSPACE"
            | "BUILD"
            | "BUILD.bazel"
            | "LICENSE"
            | "LICENSE.txt"
            | "COPYING"
            | "NOTICE"
            | "README"
            | "CHANGELOG"
    )
}

fn detect_text(path: &Path, fast: bool, size: u64, ext: &str, plan: &FileEnumerationPlan) -> bool {
    if plan.force_binary_exts.iter().any(|pattern| pattern.eq_ignore_ascii_case(ext)) {
        return false;
    }
    if plan.force_text_exts.iter().any(|pattern| pattern.eq_ignore_ascii_case(ext)) {
        return true;
    }

    // Short-circuit common binary extensions to avoid unnecessary IO and
    // reduce false positives on quick sniffing large files.
    if is_definitely_binary_ext(ext) {
        return false;
    }
    if is_definitely_text_ext(ext) {
        return true;
    }

    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        if is_definitely_text_name(name) {
            return true;
        }
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
        
        // Platform-aware normalization for sorting and deduplication.
        // Use the platform abstraction to handle case sensitivity differences.
        use crate::platform::{default_path_normalizer, PathNormalizer};
        let normalizer = default_path_normalizer();
        
        #[cfg(windows)]
        {
            // Windows: always case-insensitive
            let mut keyed: Vec<(_, FileEntryDto)> =
                walk_entries.into_iter().map(|e| (normalizer.normalize(&e.path), e)).collect();
            keyed.sort_by(|a, b| a.0.cmp(&b.0));
            keyed.dedup_by(|a, b| a.0 == b.0);
            walk_entries = keyed.into_iter().map(|(_, e)| e).collect();
        }
        #[cfg(not(windows))]
        {
            if plan.case_insensitive_dedup {
                // User requested case-insensitive dedup on case-sensitive filesystem
                let mut keyed: Vec<(String, FileEntryDto)> =
                    walk_entries.into_iter().map(|e| (e.path.to_string_lossy().to_lowercase(), e)).collect();
                keyed.sort_by(|a, b| a.0.cmp(&b.0));
                keyed.dedup_by(|a, b| a.0 == b.0);
                walk_entries = keyed.into_iter().map(|(_, e)| e).collect();
            } else {
                // Use platform-appropriate normalization
                walk_entries.sort_by_cached_key(|e| normalizer.normalize(&e.path));
                walk_entries.dedup_by(|a, b| a.path == b.path);
            }
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
        use crate::platform::{default_path_normalizer, PathNormalizer};
        let normalizer = default_path_normalizer();
        
        #[cfg(windows)]
        {
            // Windows: always case-insensitive
            let mut seen: HashSet<_> = HashSet::new();
            paths.retain(|p| seen.insert(normalizer.normalize(p)));
        }
        #[cfg(not(windows))]
        {
            if plan.case_insensitive_dedup {
                let mut seen: HashSet<String> = HashSet::new();
                paths.retain(|p| seen.insert(p.to_string_lossy().to_lowercase()));
            } else {
                let mut seen: HashSet<_> = HashSet::new();
                paths.retain(|p| seen.insert(normalizer.normalize(p)));
            }
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
            let is_text = detect_text(&path, plan.fast_text_detect, light.size, &light.ext, plan);
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
    plan: &FileEnumerationPlan,
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
    build_entry_from_path_if_matches(path, matcher, include_hidden, fast_text, follow_links, plan)
}

fn build_entry_from_path_if_matches(
    path: PathBuf,
    matcher: &PlanMatcher,
    include_hidden: bool,
    fast_text: bool,
    follow_links: bool,
    plan: &FileEnumerationPlan,
) -> Option<FileEntryDto> {
    if !include_hidden && is_hidden(&path) {
        return None;
    }
    match build_meta_light(&path, follow_links) {
        Some(light) if matcher.matches_file_light(&path, &light) => {
            let is_text = detect_text(&path, fast_text, light.size, &light.ext, plan);
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
    builder.git_ignore(plan.respect_gitignore);
    builder.git_global(plan.respect_gitignore);
    builder.git_exclude(plan.respect_gitignore);
    builder.ignore(plan.respect_gitignore);

    if let Some(depth) = plan.max_depth {
        builder.max_depth(Some(depth));
    }
    if let Some(thread_count) = plan.threads {
        if thread_count > 0 {
            builder.threads(thread_count);
        }
    }

    if plan.use_ignore_overrides && (!plan.overrides_include.is_empty() || !plan.overrides_exclude.is_empty())
    {
        let mut ob = OverrideBuilder::new(root);
        for pattern in &plan.overrides_include {
            let pat = if pattern.starts_with('!') { pattern.clone() } else { format!("!{}", pattern) };
            if let Err(err) = ob.add(&pat) {
                warn_msg(&format!("invalid include override '{}': {}", pattern, err));
            }
        }
        for pattern in &plan.overrides_exclude {
            if let Err(err) = ob.add(pattern) {
                warn_msg(&format!("invalid exclude override '{}': {}", pattern, err));
            }
        }
        match ob.build() {
            Ok(overrides) => {
                builder.overrides(overrides);
            }
            Err(err) => warn_msg(&format!("building overrides failed for {}: {}", root.display(), err)),
        }
    }

    let matcher_for_dirs = Arc::clone(matcher);
    let include_hidden = plan.include_hidden;
    let no_default_prune = plan.no_default_prune;
    let follow_links = plan.follow_links;
    
    // Lightweight visited set to avoid following directory symlink loops when
    // follow_links is enabled. Use platform-appropriate tracking.
    use crate::platform::DirectoryLoopDetector;
    let visited_dirs: Arc<Mutex<DirectoryLoopDetector>> = Arc::new(Mutex::new(DirectoryLoopDetector::new()));
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
                // we've already visited the same directory using the platform-
                // appropriate detector (inode on Unix, canonical path elsewhere).
                if follow_links {
                    let mut detector = visited_dirs_cloned.lock().unwrap();
                    if !detector.visit(entry.path(), maybe_md.as_ref()) {
                        return false; // already visited
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
    let fast_text = plan.fast_text_detect;
    let follow_links = plan.follow_links;

    // Use a thread-local buffer to reduce mutex contention: collect entries in a
    // local Vec and flush them to the shared vector in batches. The
    // LocalCollector ensures leftover items are flushed when the thread exits.

    let plan_arc = Arc::new(plan.clone());
    builder.build_parallel().run({
        let plan_arc = Arc::clone(&plan_arc);
        let matcher = Arc::clone(&matcher_ref);
        let shared = Arc::clone(&entries_clone);
        move || {
            let matcher = Arc::clone(&matcher);
            let shared = Arc::clone(&shared);
            let plan_ref = Arc::clone(&plan_arc);
            let mut collector = LocalCollector::with_capacity(shared);
            Box::new(move |result| {
                if let Some(dto) = try_build_entry_from_result(
                    result,
                    &matcher,
                    include_hidden,
                    fast_text,
                    follow_links,
                    plan_ref.as_ref(),
                ) {
                    collector.buf.push(dto);
                    if collector.buf.len() >= FLUSH_THRESHOLD {
                        let mut guard = collector.shared.lock().unwrap();
                        guard.append(&mut collector.buf);
                    }
                }
                ignore::WalkState::Continue
            })
        }
    });

    // merge local entries into main vector
    let mut guard = entries_ref.lock().unwrap();
    let collected = std::mem::take(&mut *guard);
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
    let mtime_local = meta.mtime.map(|dt| dt.with_timezone(&Local));
    FileEntryDto::new(path, meta.is_text, meta.size, meta.ext, meta.name, mtime_local)
}

fn read_files_from_lines(path: &Path) -> Result<Vec<PathBuf>> {
    let reader = FileReader::open_buffered(path)
        .map_err(|source| InfrastructureError::FileRead { path: path.to_path_buf(), source })?;
    let mut files = Vec::new();
    let mut is_first = true;
    let base = path.parent().unwrap_or(Path::new(""));
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
            let candidate = Path::new(s);
            files.push(if candidate.is_absolute() { candidate.to_path_buf() } else { base.join(candidate) });
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
    let base = path.parent().unwrap_or(Path::new(""));
    Ok(buf
        .split(|&b| b == 0)
        .filter_map(|chunk| {
            if chunk.is_empty() {
                return None;
            }
            // Convert raw bytes to PathBuf preserving non-UTF-8 paths on Unix.
            let candidate = path_from_bytes(chunk);
            Some(if candidate.is_absolute() { candidate } else { base.join(candidate) })
        })
        .collect())
}

fn collect_git_files(roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in roots {
        let output = std::process::Command::new("git")
            .args([
                "-c",
                "core.quotepath=false",
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
                "--",
                ".",
            ])
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
    Ok(files)
}

// Convert raw bytes to a PathBuf with platform-aware handling. This avoids
// lossy conversions for non-UTF-8 paths on Unix while providing a reasonable
// fallback on Windows (git on Windows typically emits UTF-8).
fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    crate::platform::path_from_bytes(bytes)
}

#[derive(Debug)]
struct FileMetadata {
    size: u64,
    mtime: Option<DateTime<Utc>>, // Stored in UTC to avoid DST issues; convert to Local when producing DTOs.
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
    let mtime = metadata.modified().ok().map(DateTime::<Utc>::from);
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
                    } else if has_known_binary_signature(s) {
                        false
                    } else if looks_like_utf16_no_bom(s) || looks_like_utf32_no_bom(s) {
                        true
                    } else {
                        memchr::memchr(0, s).is_none()
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

fn looks_like_utf16_no_bom(s: &[u8]) -> bool {
    if s.len() < 6 {
        return false;
    }
    let n = s.len().min(1024);
    let mut nul_even = 0usize;
    let mut nul_odd = 0usize;
    let mut ascii_like = 0usize;
    for (i, &b) in s[..n].iter().enumerate() {
        if b == 0 {
            if (i & 1) == 0 {
                nul_even += 1;
            } else {
                nul_odd += 1;
            }
        } else if b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b <= 0x7E) {
            ascii_like += 1;
        }
    }
    let nul_total = nul_even + nul_odd;
    if nul_total < n / 6 {
        return false;
    }
    let skew = nul_even.max(nul_odd) as f32 / (nul_total as f32 + f32::EPSILON);
    let ascii_ratio = ascii_like as f32 / n as f32;
    skew >= 0.8 && ascii_ratio >= 0.4
}

/// Detect UTF-32 (LE/BE) without BOM by looking for a single lane containing
/// ASCII-like bytes and the remaining three lanes mostly zeros.
fn looks_like_utf32_no_bom(s: &[u8]) -> bool {
    if s.len() < 8 {
        return false;
    }
    let n = s.len().min(1024);
    let mut lane_total = [0usize; 4];
    let mut nul_lane = [0usize; 4];
    let mut ascii_lane = [0usize; 4];
    for (i, &b) in s[..n].iter().enumerate() {
        let lane = i & 3;
        lane_total[lane] += 1;
        if b == 0 {
            nul_lane[lane] += 1;
        } else if b == b'\n' || b == b'\r' || b == b'\t' || (b >= 0x20 && b <= 0x7E) {
            ascii_lane[lane] += 1;
        }
    }
    let (ascii_lane_idx, &ascii_max) = ascii_lane.iter().enumerate().max_by_key(|(_, v)| *v).unwrap();
    if ascii_max == 0 {
        return false;
    }
    // Other three lanes should be mostly zeros.
    for lane in 0..4 {
        if lane == ascii_lane_idx {
            continue;
        }
        let lane_len = lane_total[lane];
        if lane_len == 0 {
            continue;
        }
        let zero_ratio = nul_lane[lane] as f32 / lane_len as f32;
        if zero_ratio < 0.8 {
            return false;
        }
    }
    let ascii_lane_len = lane_total[ascii_lane_idx];
    if ascii_lane_len == 0 {
        return false;
    }
    let ascii_ratio = ascii_max as f32 / ascii_lane_len as f32;
    ascii_ratio >= 0.4
}

fn strict_text_check(path: &Path) -> bool {
    match FileReader::open(path) {
        Ok(mut file) => {
            // Stream the file in fixed-size chunks to avoid allocating for
            // very large files while searching for NUL bytes.
            let mut buf = [0u8; 64 * 1024];
            let mut first = true;
            let mut utf16_hint = false;
            let mut utf32_hint = false;
            loop {
                match file.read(&mut buf) {
                    Ok(0) => return true, // EOF reached, no NUL seen
                    Ok(n) => {
                        let slice = &buf[..n];
                        if first {
                            first = false;
                            if slice.starts_with(&[0xEF, 0xBB, 0xBF])
                                || slice.starts_with(&[0xFF, 0xFE])
                                || slice.starts_with(&[0xFE, 0xFF])
                                || slice.starts_with(&[0xFF, 0xFE, 0x00, 0x00])
                                || slice.starts_with(&[0x00, 0x00, 0xFE, 0xFF])
                            {
                                return true;
                            }
                            if has_known_binary_signature(slice) {
                                return false;
                            }
                            utf16_hint = looks_like_utf16_no_bom(slice);
                            utf32_hint = looks_like_utf32_no_bom(slice);
                        }
                        if memchr::memchr(0, slice).is_some() {
                            if utf16_hint
                                || utf32_hint
                                || looks_like_utf16_no_bom(slice)
                                || looks_like_utf32_no_bom(slice)
                            {
                                continue;
                            }
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
    exclude_dirs_only: Vec<GlobMatcher>,
    ext_filters: HashSet<String>,
    size_min: Option<u64>,
    size_max: Option<u64>,
    mtime_since: Option<DateTime<Utc>>,
    mtime_until: Option<DateTime<Utc>>,
    override_include_paths: Vec<GlobMatcher>,
    override_exclude_paths: Vec<GlobMatcher>,
    use_overrides: bool,
}

#[allow(dead_code)]
impl PlanMatcher {
    fn new(plan: &FileEnumerationPlan) -> Result<Self> {
        // Normalize size and mtime ranges (defensive: swap when reversed)
        let (lo, hi) = normalize_size_range(plan.size_range);
        let since_utc = plan.mtime_since.map(|dt| dt.with_timezone(&Utc));
        let until_utc = plan.mtime_until.map(|dt| dt.with_timezone(&Utc));
        let (since, until) = normalize_mtime_range(since_utc, until_utc);
        let override_include_paths = compile_patterns(&plan.overrides_include)?;
        let override_exclude_paths = compile_patterns(&plan.overrides_exclude)?;
        // Normalize pattern strings on Windows (replace backslashes) so that
        // path-like detection (contains '/') behaves as expected.
        #[cfg(windows)]
        let include_pattern_strings =
            plan.include_patterns.iter().map(|p| p.replace('\\', "/")).collect::<Vec<_>>();
        #[cfg(not(windows))]
        let include_pattern_strings = plan.include_patterns.clone();
        #[cfg(windows)]
        let exclude_pattern_strings =
            plan.exclude_patterns.iter().map(|p| p.replace('\\', "/")).collect::<Vec<_>>();
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
            exclude_dirs_only: compile_patterns(&plan.exclude_dirs_only)?,
            ext_filters: plan.ext_filters.iter().map(|ext| ext.to_lowercase()).collect(),
            size_min: lo,
            size_max: hi,
            mtime_since: since,
            mtime_until: until,
            override_include_paths,
            override_exclude_paths,
            use_overrides: plan.use_ignore_overrides,
        })
    }

    fn should_visit_dir(&self, path: &Path, include_hidden: bool, no_default_prune: bool) -> bool {
        if !include_hidden && is_hidden(path) {
            return false;
        }

        if !no_default_prune && let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            #[cfg(windows)]
            let hit = DEFAULT_PRUNE_DIRS.iter().any(|d| d.eq_ignore_ascii_case(name));
            #[cfg(not(windows))]
            let hit = DEFAULT_PRUNE_DIRS.contains(&name);
            if hit {
                return false;
            }
        }

        if self.exclude_dirs_only.iter().any(|matcher| matcher.is_match(path)) {
            return false;
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

        if self.use_overrides {
            if self.override_include_paths.iter().any(|m| m.is_match(path)) {
                return true;
            }
            if self.override_exclude_paths.iter().any(|m| m.is_match(path)) {
                return false;
            }
        }

        let include_by_name = !self.include_patterns.is_empty() && self.include_patterns_match(path, fname);
        let include_by_path =
            !self.include_paths.is_empty() && self.include_paths.iter().any(|m| m.is_match(path));
        let include_filters_active = !self.include_patterns.is_empty() || !self.include_paths.is_empty();

        if include_filters_active && !(include_by_name || include_by_path) {
            return false;
        }

        if self.exclude_patterns_match(path, fname) {
            return false;
        }

        if self.exclude_paths.iter().any(|m| m.is_match(path)) {
            return false;
        }

        if self.exclude_dirs.iter().any(|m| m.is_match(path)) {
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
    if let (Some(a), Some(b)) = (lo, hi)
        && a > b
    {
        std::mem::swap(&mut lo, &mut hi);
    }
    (lo, hi)
}

// Normalize mtime range (swap when reversed). Extracted for clarity and testability.
fn normalize_mtime_range(
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
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
                GlobBuilder::new(pattern)
                    .case_insensitive(true)
                    .build()
                    .map_err(|err| {
                        InfrastructureError::OutputError(format!("invalid glob pattern '{pattern}': {err}"))
                    })
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
                    .map_err(|err| {
                        InfrastructureError::OutputError(format!("invalid glob pattern '{pattern}': {err}"))
                    })
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
        let mut plan = FileEnumerationPlan::new();
        plan.include_hidden = true;
        plan.no_default_prune = true;
        plan.fast_text_detect = true;
        plan
    }

    fn make_metadata(ext: &str, size: u64) -> FileMetadata {
        FileMetadata {
            size,
            mtime: None,
            is_text: true,
            ext: ext.to_lowercase(),
            name: format!("file.{ext}"),
        }
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
    fn materialise_paths_respects_exclude_dirs() {
        let dir = tempdir().expect("temp dir");
        let build_dir = dir.path().join("build");
        std::fs::create_dir(&build_dir).expect("create build dir");
        let artifact = build_dir.join("out.txt");
        std::fs::write(&artifact, "artifact").expect("write artifact");

        let mut plan = base_plan();
        plan.case_insensitive_dedup = true;
        plan.exclude_dirs = vec!["**/build/**".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher");

        let entries = materialise_paths(vec![artifact.clone()], &plan, &matcher).expect("materialise");
        assert!(
            entries.is_empty(),
            "expected explicit list honouring exclude_dirs to drop {}",
            artifact.display()
        );
    }

    #[test]
    fn materialise_paths_allows_explicit_entries_for_exclude_dirs_only() {
        let dir = tempdir().expect("temp dir");
        let generated_dir = dir.path().join("generated");
        std::fs::create_dir(&generated_dir).expect("create generated dir");
        let artifact = generated_dir.join("data.txt");
        std::fs::write(&artifact, "artifact").expect("write artifact");

        let mut plan = base_plan();
        plan.exclude_dirs_only = vec!["**/generated/**".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher");

        assert!(
            !matcher.should_visit_dir(
                &generated_dir,
                /*include_hidden=*/ true,
                /*no_default_prune=*/ true
            ),
            "exclude_dirs_only should prevent traversal into matching directories"
        );

        let entries =
            materialise_paths(vec![artifact.clone()], &plan, &matcher).expect("materialise succeeds");
        assert_eq!(
            entries.len(),
            1,
            "explicit paths should still materialise when excluded only for traversal"
        );
        assert_eq!(entries[0].path, artifact);
    }

    #[cfg(not(windows))]
    #[test]
    fn materialise_paths_case_insensitive_dedup_when_enabled() {
        let dir = tempdir().expect("temp dir");
        let upper = dir.path().join("A.TXT");
        let lower = dir.path().join("a.txt");
        std::fs::write(&upper, "hello").expect("write upper");
        std::fs::write(&lower, "world").expect("write lower");

        let mut plan = base_plan();
        plan.case_insensitive_dedup = true;
        let matcher = PlanMatcher::new(&plan).expect("build matcher");
        let paths = vec![upper.clone(), lower];
        let entries = materialise_paths(paths, &plan, &matcher).expect("materialise succeeds");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, upper);
    }

    #[test]
    fn gitignore_respect_can_be_overridden() {
        let dir = tempdir().expect("temp dir");
        let dist_dir = dir.path().join("dist");
        std::fs::create_dir(&dist_dir).expect("create dist");
        let dist_file = dist_dir.join("app.js");
        std::fs::write(&dist_file, "console.log('hi');").expect("write dist file");

        let mut plan = base_plan();
        plan.exclude_paths = vec!["**/dist/**".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher");
        let entries =
            materialise_paths(vec![dist_file.clone()], &plan, &matcher).expect("materialise excludes");
        assert!(entries.is_empty(), "expected exclude_paths to hide {}", dist_file.display());

        plan.use_ignore_overrides = true;
        plan.overrides_include = vec!["**/dist/**".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher with overrides");
        let entries =
            materialise_paths(vec![dist_file.clone()], &plan, &matcher).expect("materialise overrides");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, dist_file);
    }

    #[test]
    fn include_paths_can_bypass_gitignore_when_requested() {
        use std::sync::Arc;

        let dir = tempdir().expect("temp dir");
        std::fs::write(dir.path().join(".gitignore"), "dist/\n").expect("write gitignore");
        let dist_dir = dir.path().join("dist");
        std::fs::create_dir(&dist_dir).expect("create dist");
        let dist_file = dist_dir.join("bundle.js");
        std::fs::write(&dist_file, "console.log('bundle');").expect("write dist file");

        let mut plan = base_plan();
        plan.roots = vec![dir.path().to_path_buf()];
        plan.include_paths = vec!["dist/**".into()];
        plan.respect_gitignore = true;

        let matcher = Arc::new(PlanMatcher::new(&plan).expect("build matcher respecting gitignore"));
        let entries = super::collect_entries_from_root(dir.path(), &plan, &matcher).expect("collect entries");
        assert!(entries.is_empty(), "gitignore-respecting plan should hide files under dist/");

        let mut plan_no_git = plan.clone();
        plan_no_git.respect_gitignore = false;
        let matcher = Arc::new(PlanMatcher::new(&plan_no_git).expect("build matcher without gitignore"));
        let entries =
            super::collect_entries_from_root(dir.path(), &plan_no_git, &matcher).expect("collect entries");
        assert_eq!(entries.len(), 1, "disabling gitignore should reveal the file");
        assert_eq!(entries[0].path, dist_file);

        let mut plan_with_override = plan.clone();
        plan_with_override.use_ignore_overrides = true;
        plan_with_override.overrides_include = vec!["dist/**".into()];
        let matcher = Arc::new(PlanMatcher::new(&plan_with_override).expect("build matcher with overrides"));
        let entries = super::collect_entries_from_root(dir.path(), &plan_with_override, &matcher)
            .expect("collect entries");
        assert_eq!(entries.len(), 1, "override include should bring gitignored files back into the walk");
        assert_eq!(entries[0].path, dist_file);
    }

    #[cfg(windows)]
    #[test]
    fn materialise_paths_preserves_order_and_dedup_case_insensitive_on_windows() {
        let dir = tempdir().expect("temp dir");
        let upper = dir.path().join("A.TXT");
        std::fs::write(&upper, "hello").expect("write upper");
        let lower = dir.path().join("a.txt");

        let plan = base_plan();
        let matcher = PlanMatcher::new(&plan).expect("build matcher");
        let paths = vec![upper.clone(), lower];
        let entries = materialise_paths(paths, &plan, &matcher).expect("materialise succeeds");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, upper);
    }

    #[test]
    fn filter_respects_include_hidden() {
        let mut plan = base_plan();
        plan.include_hidden = false;
        let m = PlanMatcher::new(&plan).unwrap();
        // .git is considered hidden (dot-prefixed)
        assert!(!m.should_visit_dir(
            Path::new(".git"),
            /*include_hidden=*/ false,
            /*no_default_prune=*/ true
        ));
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

    #[test]
    fn include_patterns_or_include_paths_is_or() {
        let mut plan = base_plan();
        plan.include_patterns = vec!["*.md".into()];
        plan.include_paths = vec!["src/**".into()];
        let m = PlanMatcher::new(&plan).unwrap();

        assert!(m.matches_name_or_path(Path::new("README.md")));
        assert!(m.matches_name_or_path(Path::new("src/lib.rs")));
        assert!(!m.matches_name_or_path(Path::new("tests/data.bin")));
    }

    #[test]
    fn exclude_only_filters_without_include() {
        let mut plan = base_plan();
        plan.exclude_paths = vec!["src/**".into()];
        let m = PlanMatcher::new(&plan).unwrap();
        assert!(!m.matches_name_or_path(Path::new("src/main.rs")));
        assert!(m.matches_name_or_path(Path::new("tests/main.rs")));
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
        use std::{os::unix::fs::symlink, sync::Arc};

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
    fn windows_patterns_are_case_insensitive() {
        let mut plan = base_plan();
        plan.include_paths = vec!["SRC/**".into()];
        let matcher = PlanMatcher::new(&plan).expect("build matcher");

        assert!(matcher.matches_name_or_path(Path::new("src\\main.rs")));
        assert!(matcher.matches_name_or_path(Path::new("SRC\\LIB.RS")));
    }

    #[cfg(windows)]
    #[test]
    fn default_prune_is_case_insensitive() {
        let plan = base_plan();
        let matcher = PlanMatcher::new(&plan).expect("build matcher");
        assert!(!matcher.should_visit_dir(
            Path::new("Node_Modules"),
            /*include_hidden=*/ true,
            /*no_default_prune=*/ false
        ));
    }

    #[cfg(windows)]
    #[test]
    fn windows_dedup_ignores_case_after_sort() {
        use std::path::PathBuf;
        let mut v = vec![
            FileEntryDto::new(PathBuf::from("SRC\\A.TXT"), false, 0, "txt".into(), "A.TXT".into(), None),
            FileEntryDto::new(PathBuf::from("src\\a.txt"), false, 0, "txt".into(), "a.txt".into(), None),
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
        let plan = base_plan();
        assert!(!detect_text(&p, /*fast*/ true, 100, "pdf", &plan));
    }

    #[test]
    fn strict_text_check_accepts_utf16_bom() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("u16.txt");
        std::fs::write(&p, [0xFF, 0xFE, b'a', 0x00]).unwrap();
        assert!(strict_text_check(&p));
    }

    #[test]
    fn quick_check_detects_java_class_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("Test.class");
        std::fs::write(&p, [0xCA, 0xFE, 0xBA, 0xBE, 0, 0, 0, 0]).unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_macho_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("mach");
        std::fs::write(&p, [0xFE, 0xED, 0xFA, 0xCF, 0, 0, 0, 0]).unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_wasm_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("module");
        std::fs::write(&p, [0, 0x61, 0x73, 0x6D, 0x01, 0, 0, 0]).unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_parquet_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("datafile");
        std::fs::write(&p, b"PAR1____").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_sqlite_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("database");
        std::fs::write(&p, b"SQLite format 3\0rest").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_git_pack_and_index_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let pack = dir.path().join("pack");
        let index = dir.path().join("index");
        std::fs::write(&pack, b"PACK....").unwrap();
        std::fs::write(&index, b"DIRC....").unwrap();
        assert!(!quick_text_check(&pack));
        assert!(!quick_text_check(&index));
    }

    #[test]
    fn quick_check_detects_avi_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("movie");
        let mut data = b"RIFF____AVI ".to_vec();
        data[4..8].copy_from_slice(&[0, 0, 0, 12]);
        std::fs::write(&p, data).unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_flv_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("video");
        std::fs::write(&p, b"FLV\x01\0\0\0").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_mp3_without_id3_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("audio");
        // MP3 frame header 0xFFFB indicates MPEG1 Layer III.
        std::fs::write(&p, [0xFF, 0xFB, 0x90, 0x64]).unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_midi_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("music");
        std::fs::write(&p, b"MThd\0\0\0\x06\0\0\0\x01\0\x60").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_binary_plist() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("Info.plist");
        std::fs::write(&p, b"bplist00........").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_detects_tiff_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let le = dir.path().join("image_le");
        std::fs::write(&le, b"II*\0").unwrap();
        assert!(!quick_text_check(&le));
        let be = dir.path().join("image_be");
        std::fs::write(&be, b"MM\0*").unwrap();
        assert!(!quick_text_check(&be));
    }

    #[test]
    fn quick_detects_asf_and_pcap_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let asf = dir.path().join("video.asf");
        std::fs::write(&asf, [0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11]).unwrap();
        assert!(!quick_text_check(&asf));

        let pcap_le = dir.path().join("capture_le");
        std::fs::write(&pcap_le, [0xD4, 0xC3, 0xB2, 0xA1]).unwrap();
        assert!(!quick_text_check(&pcap_le));

        let pcap_be = dir.path().join("capture_be");
        std::fs::write(&pcap_be, [0xA1, 0xB2, 0xC3, 0xD4]).unwrap();
        assert!(!quick_text_check(&pcap_be));

        let pcapng = dir.path().join("capture_ng");
        std::fs::write(&pcapng, [0x0A, 0x0D, 0x0D, 0x0A]).unwrap();
        assert!(!quick_text_check(&pcapng));

        let psd = dir.path().join("image.psd");
        std::fs::write(&psd, b"8BPS....").unwrap();
        assert!(!quick_text_check(&psd));
    }

    #[test]
    fn quick_check_detects_zip_central_dir_and_bigtiff() {
        let dir = tempfile::tempdir().unwrap();
        let zip_cd = dir.path().join("zip_cd");
        std::fs::write(&zip_cd, b"PK\x01\x02____").unwrap();
        assert!(!quick_text_check(&zip_cd));

        let bt_le = dir.path().join("bigtiff_le");
        std::fs::write(&bt_le, b"II+\0").unwrap();
        assert!(!quick_text_check(&bt_le));

        let bt_be = dir.path().join("bigtiff_be");
        std::fs::write(&bt_be, b"MM\0+").unwrap();
        assert!(!quick_text_check(&bt_be));
    }

    #[test]
    fn quick_detects_woff_fonts_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let woff = dir.path().join("font.woff");
        std::fs::write(&woff, b"wOFF\x00\x00").unwrap();
        assert!(!quick_text_check(&woff));
        let woff2 = dir.path().join("font.woff2");
        std::fs::write(&woff2, b"wOF2\x00\x00").unwrap();
        assert!(!quick_text_check(&woff2));
    }

    #[test]
    fn quick_and_strict_detect_tar_without_extension() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("archive");
        let mut header = vec![0u8; 512];
        header[257..263].copy_from_slice(b"ustar\0");
        std::fs::write(&p, header).unwrap();
        assert!(!quick_text_check(&p));
        let plan = base_plan();
        assert!(!detect_text(&p, /*fast*/ false, 512, "", &plan));
    }

    #[test]
    fn quick_check_detects_ds_store_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join(".DS_Store");
        std::fs::write(&p, b"Bud1xxxx").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_icns_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("AppIcon");
        std::fs::write(&p, b"icns____").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_flac_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("track");
        std::fs::write(&p, b"fLaC\x00\x00\x00\x22").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_bmp_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("image");
        std::fs::write(&p, b"BMabc").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_aiff_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("aiff");
        let mut data = b"FORM____AIFF".to_vec();
        data[4..8].copy_from_slice(&[0, 0, 0, 12]);
        std::fs::write(&p, data).unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_check_detects_ar_archive_as_binary() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("lib");
        std::fs::write(&p, b"!<arch>\nxxxx").unwrap();
        assert!(!quick_text_check(&p));
    }

    #[test]
    fn quick_and_strict_accept_utf16_without_bom() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("u16_nobom.txt");
        std::fs::write(&p, [b'a', 0, b'b', 0, b'\n', 0]).unwrap();
        assert!(quick_text_check(&p));
        assert!(strict_text_check(&p));
    }

    #[test]
    fn quick_and_strict_accept_utf32le_without_bom() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("u32_nobom.txt");
        // "ab\n" encoded as UTF-32LE without BOM.
        std::fs::write(&p, [0x61, 0, 0, 0, 0x62, 0, 0, 0, 0x0A, 0, 0, 0]).unwrap();
        assert!(quick_text_check(&p));
        assert!(strict_text_check(&p));
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
        let plan = base_plan();
        assert!(detect_text(&p, /*fast*/ false, LARGE_TEXT_SNIFF_THRESHOLD + 1, "", &plan));
    }

    #[test]
    fn empty_file_is_text() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("empty.txt");
        std::fs::write(&p, &[] as &[u8]).unwrap();
        let plan = base_plan();
        assert!(detect_text(&p, /*fast*/ false, 0, "txt", &plan));
    }

    #[test]
    fn definitely_text_extension_short_circuits() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("huge.log");
        std::fs::write(&p, "x").unwrap();
        let plan = base_plan();
        assert!(detect_text(&p, /*fast*/ false, LARGE_TEXT_SNIFF_THRESHOLD + 10, "log", &plan));
    }

    #[test]
    fn definitely_text_by_filename_without_ext() {
        let dir = tempfile::tempdir().unwrap();
        let docker = dir.path().join("Dockerfile");
        std::fs::write(&docker, "FROM alpine").unwrap();
        let plan = base_plan();
        assert!(detect_text(&docker, /*fast*/ false, 10, "", &plan));

        let mk = dir.path().join("Makefile");
        std::fs::write(&mk, "all:\n\t@echo ok\n").unwrap();
        assert!(detect_text(&mk, /*fast*/ true, 10, "", &plan));

        let license = dir.path().join("LICENSE");
        std::fs::write(&license, "MIT").unwrap();
        assert!(detect_text(&license, /*fast*/ true, 6, "", &plan));

        let readme = dir.path().join("README");
        std::fs::write(&readme, "# hi").unwrap();
        assert!(detect_text(&readme, /*fast*/ false, 6, "", &plan));
    }

    #[test]
    fn force_text_and_binary_extension_overrides_apply() {
        let dir = tempfile::tempdir().unwrap();
        let p_bin = dir.path().join("data.txt");
        std::fs::write(&p_bin, b"\0\0\0").unwrap();
        let mut plan = base_plan();
        plan.force_binary_exts = vec!["txt".into()];
        assert!(!detect_text(&p_bin, /*fast*/ false, 3, "txt", &plan));

        let p_text = dir.path().join("image.bin");
        std::fs::write(&p_text, "hello").unwrap();
        plan.force_binary_exts.clear();
        plan.force_text_exts = vec!["bin".into()];
        assert!(detect_text(&p_text, /*fast*/ true, 5, "bin", &plan));
    }

    #[test]
    fn normalize_size_range_swaps() {
        assert_eq!(normalize_size_range((Some(20), Some(10))), (Some(10), Some(20)));
    }

    #[test]
    fn files_from_strips_bom_and_resolves_relative_to_list_dir() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();

        let list = sub.join("list.txt");
        std::fs::write(&list, "\u{feff}a.txt\nb.txt\r\n\n").unwrap();
        let got = read_files_from_lines(&list).expect("read files");
        assert_eq!(got, vec![sub.join("a.txt"), sub.join("b.txt")]);
    }

    #[test]
    fn files_from_null_resolves_relative_to_list_dir() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("sub");
        std::fs::create_dir(&sub).unwrap();

        let list = sub.join("list.txt");
        let data = b"a.txt\0b/c.txt\0\0";
        std::fs::write(&list, data).unwrap();
        let got = read_files_from_null(&list).expect("read files");
        assert_eq!(got, vec![sub.join("a.txt"), sub.join("b/c.txt")]);
    }

    #[test]
    fn strict_mode_detects_pdf_without_extension() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("document");
        std::fs::write(&p, b"%PDF-1.7\n%...").unwrap();
        let plan = base_plan();
        assert!(!detect_text(&p, /*fast*/ false, 100, "", &plan));
    }
}

// Hidden-file detection is platform-specific: Windows uses attributes, macOS has UF_HIDDEN,
// and other platforms rely on dot-prefixed names.
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
    "bazel-testlogs",
    ".gradle",
    ".nyc_output",
    ".coverage",
    ".cargo",
    ".yarn",
    ".pnpm-store",
    ".sbt",
    ".stack-work",
    ".ipynb_checkpoints",
    ".ruff_cache",
    ".tox",
    "logs",
    "out",
    "obj",
];
#[inline]
fn has_known_binary_signature(s: &[u8]) -> bool {
    s.starts_with(b"%PDF-")
        || s.starts_with(b"PK\x03\x04")
        || s.starts_with(b"PK\x05\x06")
        || s.starts_with(b"PK\x01\x02")
        || s.starts_with(b"!<arch>\n")
        || s.starts_with(b"MZ")
        || s.starts_with(b"\x7FELF")
        || s.starts_with(&[0xFE, 0xED, 0xFA, 0xCE])
        || s.starts_with(&[0xCE, 0xFA, 0xED, 0xFE])
        || s.starts_with(&[0xFE, 0xED, 0xFA, 0xCF])
        || s.starts_with(&[0xCF, 0xFA, 0xED, 0xFE])
        || s.starts_with(b"MSCF")
        || s.starts_with(b"\0asm")
        || s.starts_with(b"PAR1")
        || s.starts_with(b"SQLite format 3\0")
        || s.starts_with(b"PACK")
        || s.starts_with(b"DIRC")
        || s.starts_with(b"Bud1")
        || s.starts_with(b"icns")
        || s.starts_with(b"wOFF")
        || s.starts_with(b"wOF2")
        || s.starts_with(b"BM")
        || s.starts_with(b"II*\0")
        || s.starts_with(b"MM\0*")
        || s.starts_with(b"II+\0")
        || s.starts_with(b"MM\0+")
        || s.starts_with(b"\x89PNG\r\n\x1A\n")
        || s.starts_with(b"\xFF\xD8\xFF")
        || s.starts_with(b"GIF87a")
        || s.starts_with(b"GIF89a")
        || s.starts_with(b"fLaC")
        || s.starts_with(b"OggS")
        || (s.starts_with(b"RIFF")
            && (s.get(8..12) == Some(b"WAVE")
                || s.get(8..12) == Some(b"WEBP")
                || s.get(8..12) == Some(b"AVI ")))
        || s.starts_with(b"FLV")
        || s.starts_with(b"MThd")
        || (s.starts_with(b"FORM") && matches!(s.get(8..12), Some(b"AIFF") | Some(b"AIFC")))
        || s.starts_with(b"caff")
        || s.get(4..8) == Some(b"ftyp")
        || s.starts_with(&[0x0A, 0x0D, 0x0D, 0x0A])
        || s.starts_with(b"\x1F\x8B")
        || s.starts_with(b"BZh")
        || s.starts_with(b"\xFD7zXZ\x00")
        || s.starts_with(b"\x28\xB5\x2F\xFD")
        || s.starts_with(b"\x04\x22\x4D\x18")
        || s.starts_with(b"7z\xBC\xAF\x27\x1C")
        || s.starts_with(b"Rar!\x1A\x07")
        || s.starts_with(&[0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11])
        || s.starts_with(&[0xD4, 0xC3, 0xB2, 0xA1])
        || s.starts_with(&[0xA1, 0xB2, 0xC3, 0xD4])
        || s.starts_with(b"ID3")
        || s.starts_with(&[0xCA, 0xFE, 0xBA, 0xBE])
        || s.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1])
        || s.starts_with(&[0x1A, 0x45, 0xDF, 0xA3])
        || s.starts_with(b"glTF")
        || s.starts_with(b"8BPS")
        || s.starts_with(b"ARROW1")
        || s.starts_with(b"FEA1")
        || s.starts_with(b"ORC\0")
        || s.starts_with(b"bplist00")
        || s.starts_with(b"Cr24")
        || s.starts_with(b"RIFX")
        || (s.len() >= 2 && s[0] == 0xFF && (s[1] & 0xE0) == 0xE0)
        || (s.len() >= 263 && (s.get(257..263) == Some(b"ustar\0") || s.get(257..263) == Some(b"ustar ")))
}
