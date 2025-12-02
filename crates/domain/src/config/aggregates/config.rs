// crates/domain/src/config/aggregates/config.rs
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use chrono::{DateTime, Local};

use crate::{
    config::{ByKey, Filters},
    options::{OutputFormat, SortKey, WatchOutput},
    value_objects::FileMeta,
};

/// Domain representation of resolved configuration options.
#[derive(Debug, Clone)]
pub struct Config {
    pub format: OutputFormat,
    pub sort_specs: Vec<(SortKey, bool)>,
    pub top_n: Option<usize>,
    pub by_modes: Vec<ByKey>,
    pub summary_only: bool,
    pub total_only: bool,
    pub by_limit: Option<usize>,
    pub filters: Filters,
    pub hidden: bool,
    pub follow: bool,
    pub use_git: bool,
    pub case_insensitive_dedup: bool,
    pub respect_gitignore: bool,
    pub use_ignore_overrides: bool,
    pub jobs: usize,
    pub no_default_prune: bool,
    pub abs_path: bool,
    pub abs_canonical: bool,
    pub trim_root: Option<PathBuf>,
    pub words: bool,
    /// SLOC (Source Lines of Code) - 空行を除外した純粋コード行数を計測するかどうか
    pub sloc: bool,
    pub count_newlines_in_chars: bool,
    pub text_only: bool,
    pub fast_text_detect: bool,
    pub files_from: Option<PathBuf>,
    pub files_from0: Option<PathBuf>,
    pub paths: Vec<PathBuf>,
    pub mtime_since: Option<DateTime<Local>>,
    pub mtime_until: Option<DateTime<Local>>,
    pub total_row: bool,
    pub progress: bool,
    pub ratio: bool,
    pub output: Option<PathBuf>,
    pub strict: bool,
    pub incremental: bool,
    pub cache_dir: Option<PathBuf>,
    pub cache_verify: bool,
    pub clear_cache: bool,
    pub watch: bool,
    pub watch_interval: Duration,
    pub watch_output: WatchOutput,
    pub compare: Option<(PathBuf, PathBuf)>,
    pub max_depth: Option<usize>,
    pub enumerator_threads: Option<usize>,
}

impl Config {
    /// Determine whether a file at `path` with the given metadata should be included.
    pub fn matches_file(&self, path: &Path, meta: &FileMeta) -> bool {
        self.matches_name_patterns(path)
            && self.matches_path_patterns(path)
            && self.matches_extension(meta)
            && self.matches_metadata(meta)
    }

    fn matches_name_patterns(&self, path: &Path) -> bool {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };

        let filters = &self.filters;

        if !filters.include_patterns.is_empty()
            && !filters.include_patterns.iter().any(|pattern| pattern.matches(file_name))
        {
            return false;
        }

        !filters.exclude_patterns.iter().any(|pattern| pattern.matches(file_name))
    }

    fn matches_path_patterns(&self, path: &Path) -> bool {
        let filters = &self.filters;

        if !filters.include_paths.is_empty()
            && !filters.include_paths.iter().any(|pattern| pattern.matches_path(path))
        {
            return false;
        }

        !filters.exclude_paths.iter().any(|pattern| pattern.matches_path(path))
    }

    fn matches_extension(&self, meta: &FileMeta) -> bool {
        let filters = &self.filters;
        if filters.ext_filters.is_empty() {
            return true;
        }

        filters.ext_filters.contains(&meta.ext)
    }

    fn matches_metadata(&self, meta: &FileMeta) -> bool {
        if !self.filters.size_range.contains(meta.size) {
            return false;
        }

        if self.mtime_since.is_some_and(|since| meta.mtime.is_some_and(|mtime| mtime < since)) {
            return false;
        }

        if self.mtime_until.is_some_and(|until| meta.mtime.is_some_and(|mtime| mtime > until)) {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::{Duration as ChronoDuration, Local};

    use super::*;
    use crate::{
        config::value_objects::{Filters, GlobPattern, SizeRange},
        value_objects::FileMeta,
    };

    fn base_config() -> Config {
        Config {
            format: OutputFormat::Table,
            sort_specs: vec![],
            top_n: None,
            by_modes: vec![],
            summary_only: false,
            total_only: false,
            by_limit: None,
            filters: Filters::default(),
            hidden: false,
            follow: false,
            use_git: false,
            case_insensitive_dedup: false,
            respect_gitignore: true,
            use_ignore_overrides: false,
            jobs: 1,
            no_default_prune: false,
            max_depth: None,
            enumerator_threads: None,
            abs_path: false,
            abs_canonical: false,
            trim_root: None,
            words: false,
            sloc: false,
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
            cache_verify: false,
            clear_cache: false,
            watch: false,
            watch_interval: std::time::Duration::from_secs(1),
            watch_output: WatchOutput::Full,
            compare: None,
        }
    }

    fn file_meta(name: &str, ext: &str, size: u64, mtime: Option<chrono::DateTime<Local>>) -> FileMeta {
        FileMeta { size, mtime, is_text: true, ext: ext.to_string(), name: name.to_string() }
    }

    #[test]
    fn matches_file_allows_when_filters_are_empty() {
        let config = base_config();
        let path = PathBuf::from("src/lib.rs");
        let meta = file_meta("lib.rs", "rs", 123, Some(Local::now()));

        assert!(config.matches_file(&path, &meta));
    }

    #[test]
    fn matches_file_returns_false_when_path_has_no_filename() {
        let config = base_config();
        let path = PathBuf::from("/");
        let meta = file_meta("", "", 0, None);

        assert!(!config.matches_file(&path, &meta));
    }

    #[test]
    fn include_patterns_must_match_file_name() {
        let mut config = base_config();
        config.filters.include_patterns = vec![GlobPattern::new("*.rs").unwrap()];

        let path = PathBuf::from("notes.txt");
        let meta = file_meta("notes.txt", "txt", 10, None);

        assert!(!config.matches_file(&path, &meta));
    }

    #[test]
    fn exclude_patterns_block_matching_files() {
        let mut config = base_config();
        config.filters.exclude_patterns = vec![GlobPattern::new("*.log").unwrap()];

        let path = PathBuf::from("error.log");
        let meta = file_meta("error.log", "log", 10, None);

        assert!(!config.matches_file(&path, &meta));
    }

    #[test]
    fn include_paths_require_match() {
        let mut config = base_config();
        config.filters.include_paths = vec![GlobPattern::new("src/**").unwrap()];

        let path = PathBuf::from("tests/main.rs");
        let meta = file_meta("main.rs", "rs", 10, None);

        assert!(!config.matches_file(&path, &meta));
    }

    #[test]
    fn exclude_paths_block_directory_matches() {
        let mut config = base_config();
        config.filters.exclude_paths = vec![GlobPattern::new("**/target/**").unwrap()];

        let path = PathBuf::from("target/tmp.rs");
        let meta = file_meta("tmp.rs", "rs", 10, None);

        assert!(!config.matches_file(&path, &meta));
    }

    #[test]
    fn extension_filters_are_respected() {
        let mut config = base_config();
        config.filters.ext_filters.insert("rs".to_string());

        let path = PathBuf::from("example.txt");
        let meta = file_meta("example.txt", "txt", 10, None);

        assert!(!config.matches_file(&path, &meta));
    }

    #[test]
    fn size_range_is_enforced() {
        let mut config = base_config();
        config.filters.size_range = SizeRange::new(Some(100), Some(200));

        let too_small = file_meta("small.rs", "rs", 50, None);
        assert!(!config.matches_file(&PathBuf::from("src/small.rs"), &too_small));

        let within_range = file_meta("ok.rs", "rs", 150, None);
        assert!(config.matches_file(&PathBuf::from("src/ok.rs"), &within_range));

        let too_large = file_meta("big.rs", "rs", 500, None);
        assert!(!config.matches_file(&PathBuf::from("src/big.rs"), &too_large));
    }

    #[test]
    fn mtime_since_and_until_filters_are_applied() {
        let mut config = base_config();
        let now = Local::now();
        config.mtime_since = Some(now);
        config.mtime_until = Some(now + ChronoDuration::seconds(60));

        let before = file_meta("old.rs", "rs", 10, Some(now - ChronoDuration::seconds(5)));
        assert!(!config.matches_file(&PathBuf::from("src/old.rs"), &before));

        let after = file_meta("future.rs", "rs", 10, Some(now + ChronoDuration::seconds(120)));
        assert!(!config.matches_file(&PathBuf::from("src/future.rs"), &after));

        let within = file_meta("current.rs", "rs", 10, Some(now + ChronoDuration::seconds(30)));
        assert!(config.matches_file(&PathBuf::from("src/current.rs"), &within));
    }
}
