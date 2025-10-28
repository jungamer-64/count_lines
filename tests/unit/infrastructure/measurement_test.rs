use count_lines_core::{
    domain::{
        config::{ByKey, Config, Filters},
        model::FileMeta,
        options::OutputFormat,
    },
    infrastructure::measurement::strategies::{measure_by_lines, measure_entire_file},
};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(prefix: &str, contents: &[u8]) -> Self {
        let base = std::env::temp_dir().join("count_lines_tests");
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
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

#[test]
fn line_based_measurement_counts_crlf_and_words() {
    let file = TempFile::new("measurement_line", b"hello\nworld\r\nlast");
    let mut config = base_config();
    config.words = true;

    let stats = measure_by_lines(&file.path, &make_meta(&file.path), &config).expect("measurement succeeded");
    assert_eq!(stats.lines, 3);
    assert_eq!(stats.chars, 14);
    assert_eq!(stats.words, Some(3));
}

#[test]
fn byte_based_measurement_counts_newlines() {
    let file = TempFile::new("measurement_byte", b"one\ntwo");
    let mut config = base_config();
    config.count_newlines_in_chars = true;
    config.words = true;

    let stats =
        measure_entire_file(&file.path, &make_meta(&file.path), &config).expect("measurement succeeded");
    assert_eq!(stats.lines, 2);
    assert_eq!(stats.chars, 7);
    assert_eq!(stats.words, Some(2));
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
