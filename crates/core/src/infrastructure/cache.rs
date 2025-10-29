use std::{
    collections::{HashMap, HashSet, hash_map::DefaultHasher},
    fs,
    hash::{Hash, Hasher},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

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
        if let Some(parent) = path.parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(InfrastructureError::FileWrite { path: parent.to_path_buf(), source: err }.into());
            }
        }
        {
            let mut file = fs::File::create(&tmp_path)
                .map_err(|err| InfrastructureError::FileWrite { path: tmp_path.clone(), source: err })?;
            file.write_all(&data)
                .map_err(|err| InfrastructureError::FileWrite { path: tmp_path.clone(), source: err })?;
            file.flush()
                .map_err(|err| InfrastructureError::FileWrite { path: tmp_path.clone(), source: err })?;
        }
        fs::rename(&tmp_path, path)
            .map_err(|err| InfrastructureError::FileWrite { path: path.clone(), source: err })?;
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
            mtime_millis: entry.meta.mtime.as_ref().map(|dt| dt.timestamp_millis()),
            lines: stats.lines as u64,
            chars: stats.chars as u64,
            words: stats.words.map(|w| w as u64),
            hash_hex,
        }
    }

    pub fn to_stats(&self, entry: &FileEntry) -> FileStats {
        FileStats::new(
            entry.path.clone(),
            self.lines as usize,
            self.chars as usize,
            self.words.map(|w| w as usize),
            &entry.meta,
        )
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

    if let Some(mut dir) = dirs::cache_dir() {
        dir.push("count_lines");
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

fn cache_file_name(config: &Config) -> String {
    let hash = workspace_hash(config);
    format!("count_lines-cache-{hash:016x}.json")
}

fn workspace_hash(config: &Config) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut paths: Vec<String> =
        config.paths.iter().map(|path| logical_absolute(path).to_string_lossy().into_owned()).collect();
    paths.sort();
    for path in paths {
        path.hash(&mut hasher);
    }
    hasher.finish()
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
            if let Err(err) = fs::remove_file(&path) {
                if err.kind() != io::ErrorKind::NotFound {
                    return Err(InfrastructureError::FileWrite { path, source: err }.into());
                }
            }
        }
        Ok(())
    }
}
