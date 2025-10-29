use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use count_lines_core::{
    domain::{
        config::{ByKey, Config, Filters},
        model::{FileEntry, FileMeta},
        options::OutputFormat,
    },
    infrastructure::{
        filesystem::services::metadata_loader::FileMetadataLoader,
        measurement::{
            measure_entries,
            strategies::{measure_by_lines, measure_entire_file},
        },
    },
};
use serde_json::Value;

struct TempDirResource {
    path: PathBuf,
}

impl TempDirResource {
    fn new(prefix: &str) -> Self {
        let base = std::env::temp_dir().join("count_lines_tests");
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string();
        let path = base.join(format!("{prefix}_{unique}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirResource {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(prefix: &str, contents: &[u8]) -> Self {
        let base = std::env::temp_dir().join("count_lines_tests");
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string();
        let path = base.join(format!("{prefix}_{unique}.tmp"));
        fs::write(&path, contents).unwrap();
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn base_config() -> Config {
    Config {
        format: OutputFormat::Json,
        sort_specs: Vec::new(),
        top_n: None,
        by_modes: vec![ByKey::Ext],
        summary_only: false,
        total_only: false,
        by_limit: None,
        filters: Filters::default(),
        hidden: false,
        follow: false,
        use_git: false,
        jobs: 1,
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
        paths: vec![PathBuf::from(".")],
        mtime_since: None,
        mtime_until: None,
        total_row: false,
        progress: false,
        ratio: false,
        output: None,
        strict: false,
        incremental: false,
        cache_dir: None,
        compare: None,
    }
}

fn make_meta(path: &PathBuf) -> FileMeta {
    let size = fs::metadata(path).unwrap().len();
    FileMeta {
        size,
        mtime: None,
        is_text: true,
        ext: path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase(),
        name: path.file_name().unwrap().to_string_lossy().into(),
    }
}

fn make_entry(path: &Path, config: &Config) -> FileEntry {
    let meta = FileMetadataLoader::build(path, config).expect("metadata loads");
    FileEntry { path: path.to_path_buf(), meta }
}

fn find_cache_file(cache_root: &Path) -> PathBuf {
    fs::read_dir(cache_root)
        .expect("cache dir readable")
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .find(|path| path.extension().is_some_and(|ext| ext == "json"))
        .expect("cache file present")
}

#[test]
fn line_based_measurement_counts_crlf_and_words() {
    let file = TempFile::new("measurement_line", b"hello\nworld\r\nlast");
    let mut config = base_config();
    config.words = true;

    let stats = measure_by_lines(&file.path, &make_meta(&file.path), &config).expect("measurement succeeded");
    assert_eq!(stats.lines().value(), 3);
    assert_eq!(stats.chars().value(), 14);
    assert_eq!(stats.words().map(|w| w.value()), Some(3));
}

#[test]
fn byte_based_measurement_counts_newlines() {
    let file = TempFile::new("measurement_byte", b"one\ntwo");
    let mut config = base_config();
    config.count_newlines_in_chars = true;
    config.words = true;

    let stats =
        measure_entire_file(&file.path, &make_meta(&file.path), &config).expect("measurement succeeded");
    assert_eq!(stats.lines().value(), 2);
    assert_eq!(stats.chars().value(), 7);
    assert_eq!(stats.words().map(|w| w.value()), Some(2));
}

#[test]
fn byte_based_measurement_respects_text_only_flag() {
    let file = TempFile::new("measurement_binary", b"text\0binary");
    let mut config = base_config();
    config.count_newlines_in_chars = true;
    config.text_only = true;

    let result = measure_entire_file(&file.path, &make_meta(&file.path), &config);
    assert!(result.is_none());
}

#[test]
fn incremental_measurement_populates_cache() {
    let workspace = TempDirResource::new("incremental_cache");
    let cache_root = workspace.path().join("cache");
    fs::create_dir_all(&cache_root).unwrap();

    let file_path = workspace.path().join("sample.txt");
    fs::write(&file_path, b"one\ntwo\n").unwrap();

    let mut config = base_config();
    config.incremental = true;
    config.cache_dir = Some(cache_root.clone());
    config.paths = vec![workspace.path().to_path_buf()];

    let entry = make_entry(&file_path, &config);
    let stats = measure_entries(vec![entry], &config).expect("incremental run succeeds");
    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].lines, 2);

    let cache_file = find_cache_file(&cache_root);
    let cache_data = fs::read_to_string(&cache_file).expect("cache readable");
    let json: Value = serde_json::from_str(&cache_data).expect("valid json");
    let entries = json["entries"].as_object().expect("entries object");
    assert_eq!(entries.len(), 1);
}

#[test]
fn incremental_measurement_updates_changed_files() {
    let workspace = TempDirResource::new("incremental_updates");
    let cache_root = workspace.path().join("cache");
    fs::create_dir_all(&cache_root).unwrap();

    let file_path = workspace.path().join("data.txt");
    fs::write(&file_path, b"line1\nline2\n").unwrap();

    let mut config = base_config();
    config.incremental = true;
    config.cache_dir = Some(cache_root.clone());
    config.paths = vec![workspace.path().to_path_buf()];

    let entry_first = make_entry(&file_path, &config);
    let first_stats = measure_entries(vec![entry_first], &config).expect("first run");
    assert_eq!(first_stats[0].lines, 2);

    fs::write(&file_path, b"line1\nline2\nline3\n").unwrap();

    let entry_second = make_entry(&file_path, &config);
    let second_stats = measure_entries(vec![entry_second], &config).expect("second run");
    assert_eq!(second_stats[0].lines, 3, "updated file should be remeasured");

    let cache_file = find_cache_file(&cache_root);
    let cache_data = fs::read_to_string(&cache_file).expect("cache readable");
    let json: Value = serde_json::from_str(&cache_data).expect("valid json");
    let entries = json["entries"].as_object().expect("entries object");
    let cached = entries.values().next().expect("entry exists");
    assert_eq!(cached["lines"].as_u64(), Some(3));
}

#[test]
fn incremental_respects_updated_filters() {
    let workspace = TempDirResource::new("incremental_filters");
    let cache_root = workspace.path().join("cache");
    fs::create_dir_all(&cache_root).unwrap();

    let file_path = workspace.path().join("short.txt");
    fs::write(&file_path, b"only\none\n").unwrap();

    let mut config = base_config();
    config.incremental = true;
    config.cache_dir = Some(cache_root.clone());
    config.paths = vec![workspace.path().to_path_buf()];

    let entry = make_entry(&file_path, &config);
    measure_entries(vec![entry], &config).expect("initial run");

    config.filters.lines_range.min = Some(10);
    let entry_again = make_entry(&file_path, &config);
    let stats = measure_entries(vec![entry_again], &config).expect("filtered run");
    assert!(stats.is_empty(), "entry should be filtered out");

    let cache_file = find_cache_file(&cache_root);
    let cache_data = fs::read_to_string(&cache_file).expect("cache readable");
    let json: Value = serde_json::from_str(&cache_data).expect("valid json");
    let entries = json["entries"].as_object().expect("entries object");
    assert!(entries.is_empty(), "filtered entries should be pruned from cache");
}
