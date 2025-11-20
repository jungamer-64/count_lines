/// tests/common/builders.rs
// テストデータ構築用ビルダー
use std::{path::PathBuf, time::Duration};

use chrono::{DateTime, Local};
use count_lines_core::{
    application::{ConfigOptions, FilterOptions},
    domain::{
        grouping::ByMode,
        model::{FileEntry, FileMeta, FileStats, FileStatsV2},
        options::{OutputFormat, SortKey, WatchOutput},
        value_objects::{
            CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
        },
    },
};

/// FileStatsのテストビルダー
#[allow(dead_code)]
pub struct FileStatsBuilder {
    path: PathBuf,
    lines: usize,
    chars: usize,
    words: Option<usize>,
    size: u64,
    mtime: Option<DateTime<Local>>,
    ext: String,
    name: String,
}

#[allow(dead_code)]
impl FileStatsBuilder {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = path.into();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("test.txt").to_string();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();

        Self { path, lines: 0, chars: 0, words: None, size: 0, mtime: None, ext, name }
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.lines = lines;
        self
    }

    pub fn chars(mut self, chars: usize) -> Self {
        self.chars = chars;
        self
    }

    pub fn words(mut self, words: usize) -> Self {
        self.words = Some(words);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn ext(mut self, ext: impl Into<String>) -> Self {
        self.ext = ext.into();
        self
    }

    pub fn mtime(mut self, mtime: DateTime<Local>) -> Self {
        self.mtime = Some(mtime);
        self
    }

    pub fn build_v2(self) -> FileStatsV2 {
        count_lines_core::domain::model::FileStatsBuilder::new(FilePath::new(self.path))
            .lines(LineCount::new(self.lines))
            .chars(CharCount::new(self.chars))
            .words(self.words.map(WordCount::new))
            .size(FileSize::new(self.size))
            .mtime(self.mtime.map(ModificationTime::new))
            .ext(FileExtension::new(self.ext))
            .name(FileName::new(self.name))
            .build()
    }

    pub fn build(self) -> FileStats {
        self.build_v2().to_legacy()
    }
}

/// FileEntryのテストビルダー
#[allow(dead_code)]
pub struct FileEntryBuilder {
    path: PathBuf,
    size: u64,
    mtime: Option<DateTime<Local>>,
    is_text: bool,
    ext: String,
    name: String,
}

#[allow(dead_code)]
impl FileEntryBuilder {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path: PathBuf = path.into();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("test.txt").to_string();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();

        Self { path, size: 0, mtime: None, is_text: true, ext, name }
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = size;
        self
    }

    pub fn text(mut self) -> Self {
        self.is_text = true;
        self
    }

    pub fn binary(mut self) -> Self {
        self.is_text = false;
        self
    }

    pub fn ext(mut self, ext: impl Into<String>) -> Self {
        self.ext = ext.into();
        self
    }

    pub fn build(self) -> FileEntry {
        FileEntry {
            path: self.path,
            meta: FileMeta {
                size: self.size,
                mtime: self.mtime,
                is_text: self.is_text,
                ext: self.ext,
                name: self.name,
            },
        }
    }
}

/// Configのテストビルダー
#[allow(dead_code)]
pub struct ConfigBuilder {
    config: count_lines_core::domain::config::Config,
}

#[allow(dead_code)]
impl ConfigBuilder {
    pub fn new() -> Self {
        use count_lines_core::domain::{
            config::{Config, Filters},
            options::{OutputFormat, WatchOutput},
        };

        Self {
            config: Config {
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
                watch_interval: Duration::from_secs(1),
                watch_output: WatchOutput::Full,
                compare: None,
            },
        }
    }

    pub fn json(mut self) -> Self {
        self.config.format = count_lines_core::domain::options::OutputFormat::Json;
        self
    }

    pub fn paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.config.paths = paths;
        self
    }

    pub fn jobs(mut self, jobs: usize) -> Self {
        self.config.jobs = jobs;
        self
    }

    pub fn words(mut self) -> Self {
        self.config.words = true;
        self
    }

    pub fn strict(mut self) -> Self {
        self.config.strict = true;
        self
    }

    pub fn incremental(mut self) -> Self {
        self.config.incremental = true;
        self
    }

    pub fn cache_dir(mut self, dir: PathBuf) -> Self {
        self.config.cache_dir = Some(dir);
        self
    }

    pub fn watch(mut self) -> Self {
        self.config.watch = true;
        self
    }

    pub fn watch_interval(mut self, secs: u64) -> Self {
        self.config.watch_interval = Duration::from_secs(secs);
        self
    }

    pub fn build(self) -> count_lines_core::domain::config::Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// ConfigOptionsのテストビルダー
#[allow(dead_code)]
pub struct ConfigOptionsBuilder {
    options: ConfigOptions,
}

#[allow(dead_code)]
impl ConfigOptionsBuilder {
    pub fn new() -> Self {
        Self {
            options: ConfigOptions {
                format: OutputFormat::Json,
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
                respect_gitignore: true,
                use_ignore_overrides: false,
                case_insensitive_dedup: false,
                max_depth: None,
                enumerator_threads: None,
                jobs: Some(1),
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
                cache_verify: false,
                clear_cache: false,
                watch: false,
                watch_interval: None,
                watch_output: WatchOutput::Full,
                compare: None,
            },
        }
    }

    pub fn format(mut self, format: OutputFormat) -> Self {
        self.options.format = format;
        self
    }

    pub fn sort_specs(mut self, specs: Vec<(SortKey, bool)>) -> Self {
        self.options.sort_specs = specs;
        self
    }

    pub fn top_n(mut self, n: usize) -> Self {
        self.options.top_n = Some(n);
        self
    }

    pub fn by(mut self, by: Vec<ByMode>) -> Self {
        self.options.by = by;
        self
    }

    pub fn summary_only(mut self) -> Self {
        self.options.summary_only = true;
        self
    }

    pub fn total_only(mut self) -> Self {
        self.options.total_only = true;
        self
    }

    pub fn by_limit(mut self, limit: usize) -> Self {
        self.options.by_limit = Some(limit);
        self
    }

    pub fn filters(mut self, filters: FilterOptions) -> Self {
        self.options.filters = filters;
        self
    }

    pub fn hidden(mut self, hidden: bool) -> Self {
        self.options.hidden = hidden;
        self
    }

    pub fn follow(mut self) -> Self {
        self.options.follow = true;
        self
    }

    pub fn use_git(mut self) -> Self {
        self.options.use_git = true;
        self
    }

    pub fn respect_gitignore(mut self, respect: bool) -> Self {
        self.options.respect_gitignore = respect;
        self
    }

    pub fn use_ignore_overrides(mut self) -> Self {
        self.options.use_ignore_overrides = true;
        self
    }

    pub fn case_insensitive_dedup(mut self) -> Self {
        self.options.case_insensitive_dedup = true;
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.options.max_depth = Some(depth);
        self
    }

    pub fn enumerator_threads(mut self, threads: usize) -> Self {
        self.options.enumerator_threads = Some(threads);
        self
    }

    pub fn jobs(mut self, jobs: usize) -> Self {
        self.options.jobs = Some(jobs);
        self
    }

    pub fn no_default_prune(mut self, no_prune: bool) -> Self {
        self.options.no_default_prune = no_prune;
        self
    }

    pub fn abs_path(mut self) -> Self {
        self.options.abs_path = true;
        self
    }

    pub fn abs_canonical(mut self) -> Self {
        self.options.abs_canonical = true;
        self
    }

    pub fn trim_root(mut self, root: PathBuf) -> Self {
        self.options.trim_root = Some(root);
        self
    }

    pub fn words(mut self, words: bool) -> Self {
        self.options.words = words;
        self
    }

    pub fn count_newlines_in_chars(mut self) -> Self {
        self.options.count_newlines_in_chars = true;
        self
    }

    pub fn text_only(mut self) -> Self {
        self.options.text_only = true;
        self
    }

    pub fn fast_text_detect(mut self) -> Self {
        self.options.fast_text_detect = true;
        self
    }

    pub fn files_from(mut self, path: PathBuf) -> Self {
        self.options.files_from = Some(path);
        self
    }

    pub fn files_from0(mut self, path: PathBuf) -> Self {
        self.options.files_from0 = Some(path);
        self
    }

    pub fn paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.options.paths = paths;
        self
    }

    pub fn mtime_since(mut self, since: DateTime<Local>) -> Self {
        self.options.mtime_since = Some(since);
        self
    }

    pub fn mtime_until(mut self, until: DateTime<Local>) -> Self {
        self.options.mtime_until = Some(until);
        self
    }

    pub fn total_row(mut self, total_row: bool) -> Self {
        self.options.total_row = total_row;
        self
    }

    pub fn progress(mut self) -> Self {
        self.options.progress = true;
        self
    }

    pub fn ratio(mut self) -> Self {
        self.options.ratio = true;
        self
    }

    pub fn output(mut self, path: PathBuf) -> Self {
        self.options.output = Some(path);
        self
    }

    pub fn strict(mut self, strict: bool) -> Self {
        self.options.strict = strict;
        self
    }

    pub fn incremental(mut self) -> Self {
        self.options.incremental = true;
        self
    }

    pub fn cache_dir(mut self, dir: PathBuf) -> Self {
        self.options.cache_dir = Some(dir);
        self
    }

    pub fn cache_verify(mut self) -> Self {
        self.options.cache_verify = true;
        self
    }

    pub fn clear_cache(mut self) -> Self {
        self.options.clear_cache = true;
        self
    }

    pub fn watch(mut self, watch: bool) -> Self {
        self.options.watch = watch;
        self
    }

    pub fn watch_interval(mut self, seconds: u64) -> Self {
        self.options.watch_interval = Some(seconds);
        self
    }

    pub fn watch_output(mut self, output: WatchOutput) -> Self {
        self.options.watch_output = output;
        self
    }

    pub fn compare(mut self, old: PathBuf, new: PathBuf) -> Self {
        self.options.compare = Some((old, new));
        self
    }

    pub fn build(self) -> ConfigOptions {
        self.options
    }
}

impl Default for ConfigOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}
