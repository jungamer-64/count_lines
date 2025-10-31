use std::{
    collections::{HashMap, HashSet},
    convert::TryFrom,
    env, fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::Xxh3;

use crate::{
    domain::{
        config::Config,
        model::{FileEntry, FileStats},
    },
    error::{InfrastructureError, Result},
    shared::path::logical_absolute,
};

const CACHE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    size: u64,
    mtime_millis: Option<i64>,
    lines: u64,
    chars: u64,
    words: Option<u64>,
    hash_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct CacheSignature {
    count_newlines_in_chars: bool,
    words: bool,
    cache_verify: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    version: u32,
    signature: CacheSignature,
    entries: HashMap<String, CacheEntry>,
}

pub struct CacheStore {
    path: Option<PathBuf>,
    signature: CacheSignature,
    entries: HashMap<String, CacheEntry>,
}

impl CacheStore {
    pub fn load(config: &Config) -> Result<Self> {
        let signature = CacheSignature::from_config(config);
        let path = resolve_cache_path(config);
        let mut entries = HashMap::new();

        if let Some(path) = path.clone() {
            match fs::read_to_string(&path) {
                Ok(contents) if !contents.is_empty() => match serde_json::from_str::<CacheFile>(&contents) {
                    Ok(file) if file.version == CACHE_VERSION && file.signature == signature => {
                        entries = file.entries;
                    }
                    Ok(_) => {
                        eprintln!(
                            "[warn] cache signature mismatch for {}; discarding cached data",
                            path.display()
                        );
                    }
                    Err(err) => {
                        eprintln!("[warn] failed to parse cache {}: {}", path.display(), err);
                    }
                },
                Ok(_) => {}
                Err(err) if err.kind() != io::ErrorKind::NotFound => {
                    eprintln!("[warn] failed to read cache {}: {}", path.display(), err);
                }
                Err(_) => {}
            }
        }

        Ok(Self { path, signature, entries })
    }

    pub fn path_key(path: &Path) -> String {
        logical_absolute(path).to_string_lossy().into_owned()
    }

    pub fn get_if_fresh(
        &self,
        key: &str,
        entry: &FileEntry,
        requires_words: bool,
        verify_hash: bool,
    ) -> Option<FileStats> {
        self.entries
            .get(key)
            .filter(|cached| cached.matches(entry, requires_words, verify_hash))
            .map(|cached| cached.to_stats(entry))
    }

    pub fn update(&mut self, key: String, entry: &FileEntry, stats: &FileStats, verify_hash: bool) {
        let hash_hex = verify_hash.then(|| hash_file(&entry.path)).flatten();
        self.entries.insert(key, CacheEntry::from_result(entry, stats, hash_hex));
    }

    pub fn prune_except(&mut self, retain: &HashSet<String>) -> Vec<String> {
        let to_remove: Vec<String> = self.entries.keys().filter(|k| !retain.contains(*k)).cloned().collect();
        for key in &to_remove {
            self.entries.remove(key);
        }
        to_remove
    }

    pub fn save(&self) -> Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };

        let file = CacheFile {
            version: CACHE_VERSION,
            signature: self.signature.clone(),
            entries: self.entries.clone(),
        };
        let data = serde_json::to_vec_pretty(&file)?;
        let tmp_path = path.with_extension("tmp");
        if let Some(parent) = path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            return Err(InfrastructureError::FileWrite { path: parent.to_path_buf(), source: err }.into());
        }
        // Acquire an exclusive lock to prevent concurrent writers from corrupting the cache.
        let lock_path = path.with_extension("lock");
        let lock_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&lock_path)
            .map_err(|err| InfrastructureError::FileWrite { path: lock_path.clone(), source: err })?;

        lock_file
            .lock_exclusive()
            .map_err(|err| InfrastructureError::FileWrite { path: lock_path.clone(), source: err })?;

        // Write atomically while holding the lock
        write_tmp_and_rename(&tmp_path, path, &data)
            .map_err(|err| InfrastructureError::FileWrite { path: tmp_path.clone(), source: err })?;

        // release lock and remove lock file if possible
        let _ = lock_file.unlock();
        let _ = fs::remove_file(&lock_path);
        Ok(())
    }
}

impl CacheEntry {
    fn matches(&self, entry: &FileEntry, requires_words: bool, verify_hash: bool) -> bool {
        if requires_words && self.words.is_none() {
            return false;
        }
        if self.size != entry.meta.size {
            return false;
        }
        if self.mtime_millis != entry.meta.mtime.as_ref().map(|dt| dt.timestamp_millis()) {
            return false;
        }
        if !verify_hash {
            return true;
        }
        match (&self.hash_hex, hash_file(&entry.path)) {
            (Some(expected), Some(actual)) => expected == &actual,
            _ => false,
        }
    }

    fn from_result(entry: &FileEntry, stats: &FileStats, hash_hex: Option<String>) -> Self {
        Self {
            size: entry.meta.size,
            mtime_millis: entry.meta.mtime.as_ref().map(chrono::DateTime::timestamp_millis),
            lines: stats.lines as u64,
            chars: stats.chars as u64,
            words: stats.words.map(|w| w as u64),
            hash_hex,
        }
    }

    pub fn to_stats(&self, entry: &FileEntry) -> FileStats {
        let lines = usize::try_from(self.lines).unwrap_or(usize::MAX);
        let chars = usize::try_from(self.chars).unwrap_or(usize::MAX);
        let words = self.words.map(|w| usize::try_from(w).unwrap_or(usize::MAX));
        FileStats::new(entry.path.clone(), lines, chars, words, &entry.meta)
    }
}

impl CacheSignature {
    fn from_config(config: &Config) -> Self {
        Self {
            count_newlines_in_chars: config.count_newlines_in_chars,
            words: config.words,
            cache_verify: config.cache_verify,
        }
    }
}

fn resolve_cache_path(config: &Config) -> Option<PathBuf> {
    if let Some(dir) = config.cache_dir.clone() {
        if ensure_dir(&dir).is_ok() {
            return Some(dir.join(cache_file_name(config)));
        }
        eprintln!("[warn] unable to create cache directory {}", dir.display());
        return None;
    }

    if let Some(cache_home) = env::var_os("XDG_CACHE_HOME") {
        let mut dir = PathBuf::from(cache_home);
        dir.push("count_lines");
        if ensure_dir(&dir).is_ok() {
            return Some(dir.join(cache_file_name(config)));
        }
    } else if let Some(home) = env::var_os("HOME") {
        let mut dir = PathBuf::from(home);
        dir.push(".cache/count_lines");
        if ensure_dir(&dir).is_ok() {
            return Some(dir.join(cache_file_name(config)));
        }
    }

    let fallback = logical_absolute(Path::new(".cache/count_lines"));
    if ensure_dir(&fallback).is_ok() {
        return Some(fallback.join(cache_file_name(config)));
    }

    None
}

fn ensure_dir(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

fn write_tmp_and_rename(tmp_path: &Path, final_path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(tmp_path)?;
    file.write_all(data)?;
    file.flush()?;
    fs::rename(tmp_path, final_path)?;
    Ok(())
}

fn cache_file_name(config: &Config) -> String {
    let hash = workspace_hash(config);
    format!("count_lines-cache-{hash:016x}.json")
}

fn workspace_hash(config: &Config) -> u64 {
    // Use a stable, cross-process hash (xxh3) so the cache filename is deterministic
    // for the same workspace paths. Avoid DefaultHasher which is intentionally
    // randomized per-process and therefore unsuitable for persistent filenames.
    let mut hasher = Xxh3::new();
    let mut paths: Vec<String> =
        config.paths.iter().map(|path| logical_absolute(path).to_string_lossy().into_owned()).collect();
    paths.sort();
    for path in paths {
        hasher.update(path.as_bytes());
        // separator to avoid accidental concatenation collisions
        hasher.update(&[0]);
    }
    hasher.digest()
}

fn hash_file(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Xxh3::new();
    let mut buf = [0u8; 8192];
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => return None,
        }
    }
    Some(format!("{:016x}", hasher.digest()))
}

impl CacheStore {
    pub fn clear(config: &Config) -> Result<()> {
        let store = Self::load(config)?;
        if let Some(path) = store.path {
            // Try to acquire lock before removing to avoid racing with a writer
            let lock_path = path.with_extension("lock");
            if let Ok(lock_file) = fs::OpenOptions::new().create(true).append(true).open(&lock_path) {
                let _ = lock_file.lock_exclusive();
                let res = fs::remove_file(&path);
                let _ = lock_file.unlock();
                let _ = fs::remove_file(&lock_path);
                if let Err(err) = res
                    && err.kind() != io::ErrorKind::NotFound
                {
                    return Err(InfrastructureError::FileWrite { path, source: err }.into());
                }
            } else {
                // fallback: try to remove directly
                if let Err(err) = fs::remove_file(&path)
                    && err.kind() != io::ErrorKind::NotFound
                {
                    return Err(InfrastructureError::FileWrite { path, source: err }.into());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        application::queries::config::{
            commands::{ConfigOptions, FilterOptions},
            queries::ConfigQueryService,
        },
        domain::options::{OutputFormat, WatchOutput},
    };

    fn make_options_with_paths(paths: Vec<&str>) -> ConfigOptions {
        ConfigOptions {
            format: OutputFormat::Table,
            sort_specs: vec![],
            top_n: None,
            by: vec![],
            summary_only: false,
            total_only: false,
            by_limit: None,
            filters: FilterOptions::default(),
            hidden: false,
            follow: false,
            use_git: false,
            jobs: None,
            no_default_prune: false,
            abs_path: false,
            abs_canonical: false,
            trim_root: None,
            words: false,
            count_newlines_in_chars: false,
            text_only: false,
            fast_text_detect: false,
            files_from: None,
            files_from0: None,
            paths: paths.into_iter().map(PathBuf::from).collect(),
            mtime_since: None,
            mtime_until: None,
            total_row: false,
            progress: false,
            ratio: false,
            output: None,
            strict: false,
            incremental: false,
            cache_dir: None,
            cache_verify: false,
            clear_cache: false,
            watch: false,
            watch_interval: None,
            watch_output: WatchOutput::Full,
            compare: None,
        }
    }

    #[test]
    fn workspace_hash_is_deterministic_for_path_order() {
        let opts1 = make_options_with_paths(vec!["./a", "./b"]);
        let opts2 = make_options_with_paths(vec!["./b", "./a"]);
        let config1 = ConfigQueryService::build(opts1).expect("build config");
        let config2 = ConfigQueryService::build(opts2).expect("build config");
        let name1 = super::cache_file_name(&config1);
        let name2 = super::cache_file_name(&config2);
        assert_eq!(name1, name2, "cache file names should match regardless of path order");
    }

    #[test]
    fn concurrent_cache_saves_are_atomic() {
        use std::thread;

        use tempfile::tempdir;

        let tmp = tempdir().expect("tempdir");
        let opts = make_options_with_paths(vec!["."]);
        let mut config = ConfigQueryService::build(opts).expect("build config");
        config.cache_dir = Some(tmp.path().to_path_buf());

        let n = 8usize;
        let mut handles = Vec::with_capacity(n);

        for i in 0..n {
            let cfg = config.clone();
            handles.push(thread::spawn(move || {
                let mut store = super::CacheStore::load(&cfg).expect("load cache");
                let path = cfg.cache_dir.as_ref().unwrap().join(format!("f_{i}.txt"));
                let meta = crate::domain::model::FileMeta {
                    size: 0,
                    mtime: None,
                    is_text: true,
                    ext: "txt".to_string(),
                    name: format!("f_{i}.txt"),
                };
                let entry = crate::domain::model::FileEntry { path: path.clone(), meta: meta.clone() };
                let stats =
                    crate::domain::model::entities::file_stats::FileStats::new(path, 0, 0, None, &meta);
                store.update(format!("k{i}"), &entry, &stats, false);
                // retry save a few times in case of transient filesystem races on some platforms
                let mut attempts = 0;
                loop {
                    match store.save() {
                        Ok(()) => break,
                        Err(_) if attempts < 5 => {
                            attempts += 1;
                            std::thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(e) => panic!("save: {e:?}"),
                    }
                }
            }));
        }

        for h in handles {
            h.join().expect("join");
        }

        let path = super::resolve_cache_path(&config).expect("resolve cache path");
        let contents = std::fs::read_to_string(path).expect("read cache");
        let parsed: serde_json::Value = serde_json::from_str(&contents).expect("parse cache");
        let entries = parsed.get("entries").and_then(|e| e.as_object()).expect("entries");
        // concurrent saves should not corrupt the cache JSON; at least one entry should exist
        assert!(!entries.is_empty());
    }
}
