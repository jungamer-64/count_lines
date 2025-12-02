use std::{cmp::Ordering, path::PathBuf};

use chrono::{DateTime, Local};
pub use file_stats_v2::{FileStats as FileStatsV2, FileStatsBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    model::FileMeta,
    value_objects::{
        CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, SlocCount,
        WordCount,
    },
};

/// 既存コード向けのレガシーFileStats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStats {
    pub path: PathBuf,
    pub lines: usize,
    pub chars: usize,
    pub words: Option<usize>,
    /// SLOC (Source Lines of Code) - 空行を除外した純粋コード行数
    #[serde(default)]
    pub sloc: Option<usize>,
    pub size: u64,
    pub mtime: Option<DateTime<Local>>,
    pub ext: String,
    pub name: String,
}

impl FileStats {
    /// 既存API互換のコンストラクタ
    pub fn new(path: PathBuf, lines: usize, chars: usize, words: Option<usize>, meta: &FileMeta) -> Self {
        Self {
            path,
            lines,
            chars,
            words,
            sloc: None,
            size: meta.size,
            mtime: meta.mtime,
            ext: meta.ext.clone(),
            name: meta.name.clone(),
        }
    }

    /// SLOC付きのコンストラクタ
    pub fn with_sloc(
        path: PathBuf,
        lines: usize,
        chars: usize,
        words: Option<usize>,
        sloc: Option<usize>,
        meta: &FileMeta,
    ) -> Self {
        Self {
            path,
            lines,
            chars,
            words,
            sloc,
            size: meta.size,
            mtime: meta.mtime,
            ext: meta.ext.clone(),
            name: meta.name.clone(),
        }
    }

    /// Value object版へ変換
    pub fn to_v2(&self) -> FileStatsV2 {
        FileStatsV2::from_legacy_ref(self)
    }

    /// Value object版からの変換
    pub fn from_v2(stats: FileStatsV2) -> Self {
        stats.to_legacy()
    }
}

impl PartialOrd for FileStats {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FileStats {
    fn cmp(&self, other: &Self) -> Ordering {
        self.lines.cmp(&other.lines)
    }
}

/// 新実装のFileStats (value object版)
mod file_stats_v2 {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FileStats {
        path: FilePath,
        lines: LineCount,
        chars: CharCount,
        words: Option<WordCount>,
        /// SLOC (Source Lines of Code) - 空行を除外した純粋コード行数
        #[serde(default)]
        sloc: Option<SlocCount>,
        size: FileSize,
        mtime: Option<ModificationTime>,
        ext: FileExtension,
        name: FileName,
    }

    impl FileStats {
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            path: FilePath,
            lines: LineCount,
            chars: CharCount,
            words: Option<WordCount>,
            sloc: Option<SlocCount>,
            size: FileSize,
            mtime: Option<ModificationTime>,
            ext: FileExtension,
            name: FileName,
        ) -> Self {
            Self { path, lines, chars, words, sloc, size, mtime, ext, name }
        }

        pub fn builder(path: FilePath) -> FileStatsBuilder {
            FileStatsBuilder::new(path)
        }

        #[inline]
        pub fn path(&self) -> &FilePath {
            &self.path
        }

        #[inline]
        pub fn lines(&self) -> LineCount {
            self.lines
        }

        #[inline]
        pub fn chars(&self) -> CharCount {
            self.chars
        }

        #[inline]
        pub fn words(&self) -> Option<WordCount> {
            self.words
        }

        #[inline]
        pub fn sloc(&self) -> Option<SlocCount> {
            self.sloc
        }

        #[inline]
        pub fn size(&self) -> FileSize {
            self.size
        }

        #[inline]
        pub fn mtime(&self) -> Option<ModificationTime> {
            self.mtime.clone()
        }

        #[inline]
        pub fn ext(&self) -> &FileExtension {
            &self.ext
        }

        #[inline]
        pub fn name(&self) -> &FileName {
            &self.name
        }

        pub fn to_legacy(&self) -> super::FileStats {
            super::FileStats {
                path: self.path.to_path_buf(),
                lines: self.lines.value(),
                chars: self.chars.value(),
                words: self.words.map(|w| w.value()),
                sloc: self.sloc.map(|s| s.value()),
                size: self.size.bytes(),
                mtime: self.mtime.as_ref().map(|m| *m.timestamp()),
                ext: self.ext.as_str().to_string(),
                name: self.name.as_str().to_string(),
            }
        }

        pub fn from_legacy(legacy: super::FileStats) -> Self {
            Self::from_legacy_ref(&legacy)
        }

        pub fn from_legacy_ref(legacy: &super::FileStats) -> Self {
            Self {
                path: FilePath::new(legacy.path.clone()),
                lines: LineCount::new(legacy.lines),
                chars: CharCount::new(legacy.chars),
                words: legacy.words.map(WordCount::new),
                sloc: legacy.sloc.map(SlocCount::new),
                size: FileSize::new(legacy.size),
                mtime: legacy.mtime.as_ref().map(|m| ModificationTime::new(*m)),
                ext: FileExtension::new(legacy.ext.clone()),
                name: FileName::new(legacy.name.clone()),
            }
        }
    }

    impl PartialOrd for FileStats {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for FileStats {
        fn cmp(&self, other: &Self) -> Ordering {
            self.lines.cmp(&other.lines)
        }
    }

    pub struct FileStatsBuilder {
        path: FilePath,
        lines: LineCount,
        chars: CharCount,
        words: Option<WordCount>,
        sloc: Option<SlocCount>,
        size: FileSize,
        mtime: Option<ModificationTime>,
        ext: FileExtension,
        name: FileName,
    }

    impl FileStatsBuilder {
        pub fn new(path: FilePath) -> Self {
            let name = path.file_name().unwrap_or_else(|| FileName::new("unknown".to_string()));
            let ext = path.extension().unwrap_or_default();

            Self {
                path,
                lines: LineCount::zero(),
                chars: CharCount::zero(),
                words: None,
                sloc: None,
                size: FileSize::zero(),
                mtime: None,
                ext,
                name,
            }
        }

        pub fn lines(mut self, lines: LineCount) -> Self {
            self.lines = lines;
            self
        }

        pub fn chars(mut self, chars: CharCount) -> Self {
            self.chars = chars;
            self
        }

        pub fn words(mut self, words: Option<WordCount>) -> Self {
            self.words = words;
            self
        }

        pub fn sloc(mut self, sloc: Option<SlocCount>) -> Self {
            self.sloc = sloc;
            self
        }

        pub fn size(mut self, size: FileSize) -> Self {
            self.size = size;
            self
        }

        pub fn mtime(mut self, mtime: Option<ModificationTime>) -> Self {
            self.mtime = mtime;
            self
        }

        pub fn ext(mut self, ext: FileExtension) -> Self {
            self.ext = ext;
            self
        }

        pub fn name(mut self, name: FileName) -> Self {
            self.name = name;
            self
        }

        pub fn build(self) -> FileStats {
            FileStats {
                path: self.path,
                lines: self.lines,
                chars: self.chars,
                words: self.words,
                sloc: self.sloc,
                size: self.size,
                mtime: self.mtime,
                ext: self.ext,
                name: self.name,
            }
        }

        pub fn build_legacy(self) -> super::FileStats {
            self.build().to_legacy()
        }
    }

    #[cfg(test)]
    mod tests {
        use std::path::PathBuf;

        use super::*;

        #[test]
        fn builder_creates_stats() {
            let stats = FileStats::builder(FilePath::new(PathBuf::from("test.rs")))
                .lines(LineCount::new(100))
                .chars(CharCount::new(500))
                .words(Some(WordCount::new(50)))
                .build();

            assert_eq!(stats.lines().value(), 100);
            assert_eq!(stats.chars().value(), 500);
            assert_eq!(stats.words().unwrap().value(), 50);
        }

        #[test]
        fn conversion_roundtrip() {
            let stats = FileStats::builder(FilePath::new(PathBuf::from("test.rs")))
                .lines(LineCount::new(10))
                .chars(CharCount::new(20))
                .words(Some(WordCount::new(5)))
                .build();

            let legacy = stats.to_legacy();
            let back = FileStats::from_legacy_ref(&legacy);

            assert_eq!(back.lines(), stats.lines());
            assert_eq!(legacy.lines, 10);
            assert_eq!(legacy.chars, 20);
            assert_eq!(legacy.words, Some(5));
        }
    }
}

pub type LegacyFileStats = FileStats;

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    #[test]
    fn legacy_to_v2_conversion() {
        let meta = FileMeta {
            size: 1024,
            mtime: Some(Local::now()),
            is_text: true,
            ext: "rs".to_string(),
            name: "test.rs".to_string(),
        };

        let legacy = FileStats::new(PathBuf::from("src/test.rs"), 10, 100, Some(20), &meta);

        let v2 = legacy.to_v2();
        assert_eq!(v2.lines().value(), 10);
        assert_eq!(v2.chars().value(), 100);
        assert_eq!(v2.words().unwrap().value(), 20);

        let roundtrip = FileStats::from_v2(v2);
        assert_eq!(roundtrip.lines, 10);
        assert_eq!(roundtrip.ext, "rs");
    }
}
