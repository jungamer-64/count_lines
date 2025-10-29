//! テストデータ構築用ビルダー

use std::path::PathBuf;

use chrono::{DateTime, Local};
use count_lines_core::domain::{
    model::{FileEntry, FileMeta, FileStats, FileStatsV2},
    value_objects::{
        CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
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
            options::OutputFormat,
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

    pub fn build(self) -> count_lines_core::domain::config::Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
