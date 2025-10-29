use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use count_lines_core::{
    domain::{
        config::{ByKey, Config, Filters},
        options::OutputFormat,
    },
    infrastructure::filesystem::services::{collect_entries, collect_walk_entries},
};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let base = std::env::temp_dir().join("count_lines_tests");
        fs::create_dir_all(&base).unwrap();
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_string();
        let path = base.join(format!("{prefix}_{unique}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn base_config(root: &Path) -> Config {
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
        no_default_prune: true,
        abs_path: false,
        abs_canonical: false,
        trim_root: None,
        words: false,
        count_newlines_in_chars: false,
        text_only: false,
        fast_text_detect: false,
        files_from: None,
        files_from0: None,
        paths: vec![root.to_path_buf()],
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

#[test]
fn collect_walk_skips_hidden_files_by_default() {
    let temp = TempDir::new("filesystem_hidden");
    let hidden = temp.path.join(".secret.txt");
    let visible = temp.path.join("visible.txt");

    fs::write(&hidden, b"hidden content").unwrap();
    fs::write(&visible, b"visible content").unwrap();

    let config = base_config(&temp.path);
    let entries = collect_walk_entries(&config).expect("walk succeeds");

    let paths: HashSet<_> = entries.into_iter().map(|e| e.path).collect();
    assert!(paths.contains(&visible));
    assert!(!paths.contains(&hidden));
}

#[test]
fn collect_walk_includes_hidden_when_enabled() {
    let temp = TempDir::new("filesystem_hidden_enabled");
    let hidden = temp.path.join(".secret.txt");
    fs::write(&hidden, b"hidden content").unwrap();

    let mut config = base_config(&temp.path);
    config.hidden = true;
    let entries = collect_walk_entries(&config).expect("walk succeeds");
    let paths: Vec<_> = entries.into_iter().map(|e| e.path).collect();
    assert!(paths.contains(&hidden));
}

#[test]
fn collect_walk_respects_extension_filters() {
    let temp = TempDir::new("filesystem_ext");
    let rs_file = temp.path.join("lib.rs");
    let txt_file = temp.path.join("notes.txt");
    fs::write(&rs_file, b"fn main() {}").unwrap();
    fs::write(&txt_file, b"plain text").unwrap();

    let mut config = base_config(&temp.path);
    config.filters.ext_filters = HashSet::from([String::from("rs")]);

    let entries = collect_walk_entries(&config).expect("walk succeeds");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, rs_file);
}

#[test]
fn collect_entries_reads_paths_from_files_from_list() {
    let temp = TempDir::new("filesystem_files_from");
    let file_a = temp.path.join("alpha.txt");
    let file_b = temp.path.join("beta.txt");
    fs::write(&file_a, b"first").unwrap();
    fs::write(&file_b, b"second").unwrap();

    let list_path = temp.path.join("files.list");
    let list_contents = format!("{}\n{}\n", file_a.display(), file_b.display());
    fs::write(&list_path, list_contents).unwrap();

    let mut config = base_config(&temp.path);
    config.files_from = Some(list_path);

    let entries = collect_entries(&config).expect("collect succeeds");
    let paths: Vec<_> = entries.into_iter().map(|entry| entry.path).collect();
    assert_eq!(paths, vec![file_a, file_b]);
}

#[test]
fn collect_entries_reads_paths_from_null_terminated_list() {
    let temp = TempDir::new("filesystem_files_from0");
    let file_a = temp.path.join("gamma.txt");
    let file_b = temp.path.join("delta.txt");
    fs::write(&file_a, b"third").unwrap();
    fs::write(&file_b, b"fourth").unwrap();

    let list_path = temp.path.join("files0.list");
    let list_contents = format!("{}\0{}\0", file_a.display(), file_b.display());
    fs::write(&list_path, list_contents).unwrap();

    let mut config = base_config(&temp.path);
    config.files_from0 = Some(list_path);

    let entries = collect_entries(&config).expect("collect succeeds");
    let paths: Vec<_> = entries.into_iter().map(|entry| entry.path).collect();
    assert_eq!(paths, vec![file_a, file_b]);
}
