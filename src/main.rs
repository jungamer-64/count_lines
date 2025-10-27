// src/main.rs
#![allow(clippy::multiple_crate_versions)]

use anyhow::{Context, Result};
use atty::Stream;
use clap::Parser;

const VERSION: &str = "2.4.2";

// =====================================================================================
// CLI
// =====================================================================================
mod cli {
    use super::util::{DateTimeArg, SizeArg};
    use clap::{Parser, ValueEnum};
    use std::path::PathBuf;

    #[derive(Debug, Clone, Copy, ValueEnum)]
    pub enum OutputFormat {
        Table,
        Csv,
        Tsv,
        Json,
        Yaml,
        Md,
        Jsonl,
    }

    #[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
    pub enum SortKey {
        Lines,
        Chars,
        Words,
        Name,
        Ext,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ByMode {
        None,
        Ext,
        Dir(usize),
        Mtime(Granularity),
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Granularity {
        Day,
        Week,
        Month,
    }

    impl std::str::FromStr for ByMode {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "ext" => Ok(Self::Ext),
                "none" => Ok(Self::None),
                _ if s.starts_with("dir") => {
                    let depth = s
                        .strip_prefix("dir=")
                        .and_then(|d| d.parse().ok())
                        .unwrap_or(1);
                    Ok(Self::Dir(depth))
                }
                _ if s.starts_with("mtime") => {
                    let gran = s.split(':').nth(1).unwrap_or("day");
                    let g = match gran {
                        "day" => Granularity::Day,
                        "week" => Granularity::Week,
                        "month" => Granularity::Month,
                        _ => return Err(format!("Unknown mtime granularity: {gran}")),
                    };
                    Ok(Self::Mtime(g))
                }
                other => Err(format!("Unknown --by mode: {other}")),
            }
        }
    }

    /// Sort specification. Example: `lines:desc,chars:desc,name`.
    #[derive(Debug, Clone)]
    pub struct SortSpec(pub Vec<(SortKey, bool)>);

    impl std::str::FromStr for SortSpec {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let specs = s
                .split(',')
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .map(Self::parse_single_spec)
                .collect::<Result<Vec<_>, _>>()?;

            if specs.is_empty() {
                return Err("empty sort spec".into());
            }
            Ok(SortSpec(specs))
        }
    }

    impl SortSpec {
        fn parse_single_spec(part: &str) -> Result<(SortKey, bool), String> {
            let (key_str, desc) = part
                .split_once(':')
                .map_or((part, false), |(k, d)| (k.trim(), matches!(d.trim(), "desc" | "DESC")));

            let key = Self::parse_sort_key(key_str)?;
            Ok((key, desc))
        }

        fn parse_sort_key(key_str: &str) -> Result<SortKey, String> {
            match key_str.to_ascii_lowercase().as_str() {
                "lines" => Ok(SortKey::Lines),
                "chars" => Ok(SortKey::Chars),
                "words" => Ok(SortKey::Words),
                "name" => Ok(SortKey::Name),
                "ext" => Ok(SortKey::Ext),
                other => Err(format!("Unknown sort key: {other}")),
            }
        }
    }

    #[derive(Parser, Debug)]
    #[command(
        name = "count_lines",
        version = crate::VERSION,
        about = "ファイル行数/文字数/単語数の集計ツール",
        long_about = Some(include_str!("../usage.txt"))
    )]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Args {
        /// 出力フォーマット
        #[arg(long, value_enum, default_value = "table")]
        pub format: OutputFormat,

        /// ソートキー（複数可, 例: lines:desc,chars:desc,name）
        #[arg(long, default_value = "lines:desc")]
        pub sort: SortSpec,

        /// 上位N件のみ表示
        #[arg(long)]
        pub top: Option<usize>,

        /// サマリ軸 (ext, dir, dir=N, mtime[:day|week|month]) — 複数可
        #[arg(long)]
        pub by: Vec<ByMode>,

        /// サマリのみ表示（一覧は出力しないが By 集計は出す）
        #[arg(long)]
        pub summary_only: bool,

        /// 合計のみ表示（一覧と By 集計は出さない）
        #[arg(long)]
        pub total_only: bool,

        /// 集計テーブルの上位N件のみ表示
        #[arg(long)]
        pub by_limit: Option<usize>,

        /// 含めるファイル名パターン
        #[arg(long)]
        pub include: Vec<String>,

        /// 除外するファイル名パターン
        #[arg(long)]
        pub exclude: Vec<String>,

        /// 含めるパスパターン
        #[arg(long)]
        pub include_path: Vec<String>,

        /// 除外するパスパターン
        #[arg(long)]
        pub exclude_path: Vec<String>,

        /// 除外ディレクトリパターン
        #[arg(long)]
        pub exclude_dir: Vec<String>,

        /// 拡張子フィルタ (カンマ区切り)
        #[arg(long)]
        pub ext: Option<String>,

        /// 最大ファイルサイズ (例: 10K, 5MiB)
        #[arg(long)]
        pub max_size: Option<SizeArg>,

        /// 最小ファイルサイズ (例: 10K, 5MiB)
        #[arg(long)]
        pub min_size: Option<SizeArg>,

        /// 最小行数
        #[arg(long)]
        pub min_lines: Option<usize>,

        /// 最大行数
        #[arg(long)]
        pub max_lines: Option<usize>,

        /// 最小文字数
        #[arg(long)]
        pub min_chars: Option<usize>,

        /// 最大文字数
        #[arg(long)]
        pub max_chars: Option<usize>,

        /// 単語数も計測
        #[arg(long)]
        pub words: bool,

        /// 最小単語数
        #[arg(long)]
        pub min_words: Option<usize>,

        /// 最大単語数
        #[arg(long)]
        pub max_words: Option<usize>,

        /// テキストファイルのみ
        #[arg(long)]
        pub text_only: bool,

        /// 高速テキスト判定（先頭 1024B, NUL 検出）。false なら厳密。
        #[arg(long, default_value_t = true)]
        pub fast_text_detect: bool,

        /// ファイル一覧を読み込む (改行区切り)
        #[arg(long)]
        pub files_from: Option<PathBuf>,

        /// ファイル一覧を読み込む (NUL 区切り)
        #[arg(long)]
        pub files_from0: Option<PathBuf>,

        /// 隠しファイルも対象
        #[arg(long)]
        pub hidden: bool,

        /// シンボリックリンクを辿る
        #[arg(long)]
        pub follow: bool,

        /// .gitignore を尊重 (git ls-files ベース)
        #[arg(long)]
        pub git: bool,

        /// 並列数
        #[arg(long)]
        pub jobs: Option<usize>,

        /// 既定の剪定を無効化
        #[arg(long)]
        pub no_default_prune: bool,

        /// 絶対パス出力（論理的：シンボリック解決なし）
        #[arg(long)]
        pub abs_path: bool,

        /// 絶対パスを実体解決（canonicalize）で出力
        #[arg(long)]
        pub abs_canonical: bool,

        /// パス先頭を削除
        #[arg(long)]
        pub trim_root: Option<PathBuf>,

        /// CSV/TSV 末尾に TOTAL 行を出力
        #[arg(long)]
        pub total_row: bool,

        /// 指定日時以降 (RFC3339 / %Y-%m-%d %H:%M:%S / %Y-%m-%d)
        #[arg(long)]
        pub mtime_since: Option<DateTimeArg>,

        /// 指定日時以前
        #[arg(long)]
        pub mtime_until: Option<DateTimeArg>,

        /// 改行も文字数に含める（直感的カウント）
        #[arg(long)]
        pub count_newlines_in_chars: bool,

        /// 進捗表示
        #[arg(long)]
        pub progress: bool,

        /// フィルタ式（例: "lines > 100 && ext == 'rs'")
        #[arg(long)]
        pub filter: Option<String>,

        /// 比率列を追加（一覧/集計で%)
        #[arg(long)]
        pub ratio: bool,

        /// 出力先ファイル（未指定は標準出力）
        #[arg(long)]
        pub output: Option<PathBuf>,

        /// 厳格モード（1件失敗で終了）。既定は警告して続行
        #[arg(long)]
        pub strict: bool,

        /// 比較: 2つの JSON を比較表示
        #[arg(long, num_args = 2)]
        pub compare: Option<Vec<PathBuf>>,

        /// 対象パス
        pub paths: Vec<PathBuf>,
    }
}

// =====================================================================================
// Config + Filters
// =====================================================================================
mod config {
    use super::cli::{ByMode, Granularity, OutputFormat, SortKey};
    use crate::util::logical_absolute;
    use anyhow::{anyhow, Result};
    use chrono::{DateTime, Local};
    use evalexpr::Node;
    use std::collections::HashSet;
    use std::path::PathBuf;

    pub type GlobPattern = glob::Pattern;

    #[derive(Debug, Default)]
    pub struct Filters {
        pub include_patterns: Vec<GlobPattern>,
        pub exclude_patterns: Vec<GlobPattern>,
        pub include_paths: Vec<GlobPattern>,
        pub exclude_paths: Vec<GlobPattern>,
        pub exclude_dirs: Vec<GlobPattern>,
        pub ext_filters: HashSet<String>,
        pub size_range: SizeRange,
        pub lines_range: Range,
        pub chars_range: Range,
        pub words_range: Range,
        pub filter_ast: Option<Node>,
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub struct SizeRange {
        pub min: Option<u64>,
        pub max: Option<u64>,
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub struct Range {
        pub min: Option<usize>,
        pub max: Option<usize>,
    }

    impl Range {
        fn new(min: Option<usize>, max: Option<usize>) -> Self { Self { min, max } }
        pub fn contains(&self, v: usize) -> bool {
            self.min.map_or(true, |m| v >= m) && self.max.map_or(true, |x| v <= x)
        }
    }

    impl SizeRange {
        fn new(min: Option<u64>, max: Option<u64>) -> Self { Self { min, max } }
        pub fn contains(&self, v: u64) -> bool {
            self.min.map_or(true, |m| v >= m) && self.max.map_or(true, |x| v <= x)
        }
    }

    impl Filters {
        pub fn from_args(args: &crate::cli::Args) -> Result<Self> {
            let filter_ast = args
                .filter
                .as_ref()
                .map(|expr| evalexpr::build_operator_tree(expr).map_err(|e| anyhow!(e)))
                .transpose()?;

            Ok(Self {
                include_patterns: crate::util::parse_patterns(&args.include)?,
                exclude_patterns: crate::util::parse_patterns(&args.exclude)?,
                include_paths: crate::util::parse_patterns(&args.include_path)?,
                exclude_paths: crate::util::parse_patterns(&args.exclude_path)?,
                exclude_dirs: crate::util::parse_patterns(&args.exclude_dir)?,
                ext_filters: Self::parse_extensions(&args.ext),
                size_range: SizeRange::new(args.min_size.map(|s| s.0), args.max_size.map(|s| s.0)),
                lines_range: Range::new(args.min_lines, args.max_lines),
                chars_range: Range::new(args.min_chars, args.max_chars),
                words_range: Range::new(args.min_words, args.max_words),
                filter_ast,
            })
        }

        fn parse_extensions(ext_arg: &Option<String>) -> HashSet<String> {
            ext_arg
                .as_ref()
                .map(|s| s.split(',').map(|e| e.trim().to_lowercase()).collect())
                .unwrap_or_default()
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum ByKey { Ext, Dir(usize), Mtime(Granularity) }

    #[derive(Debug)]
    #[allow(clippy::struct_excessive_bools)]
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
        pub jobs: usize,
        pub no_default_prune: bool,
        pub abs_path: bool,
        pub abs_canonical: bool,
        pub trim_root: Option<PathBuf>,
        pub words: bool,
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
        pub compare: Option<(PathBuf, PathBuf)>,
    }

    impl TryFrom<crate::cli::Args> for Config {
        type Error = anyhow::Error;
        fn try_from(args: crate::cli::Args) -> Result<Self> {
            let filters = Filters::from_args(&args)?;
            let jobs = args.jobs.unwrap_or_else(num_cpus::get).max(1);

            let paths = if args.paths.is_empty() { vec![PathBuf::from(".")] } else { args.paths };

            let by_modes = args
                .by
                .into_iter()
                .filter(|b| !matches!(b, ByMode::None))
                .map(Self::convert_by_mode)
                .collect();

            let compare = args.compare.and_then(|v| (v.len() == 2).then(|| (v[0].clone(), v[1].clone())));

            Ok(Self {
                format: args.format,
                sort_specs: args.sort.0,
                top_n: args.top,
                by_modes,
                summary_only: args.summary_only,
                total_only: args.total_only,
                by_limit: args.by_limit,
                filters,
                hidden: args.hidden,
                follow: args.follow,
                use_git: args.git,
                jobs,
                no_default_prune: args.no_default_prune,
                abs_path: args.abs_path,
                abs_canonical: args.abs_canonical,
                trim_root: args.trim_root.map(|p| logical_absolute(&p)),
                words: args.words,
                count_newlines_in_chars: args.count_newlines_in_chars,
                text_only: args.text_only,
                fast_text_detect: args.fast_text_detect,
                files_from: args.files_from,
                files_from0: args.files_from0,
                paths,
                mtime_since: args.mtime_since.map(|d| d.0),
                mtime_until: args.mtime_until.map(|d| d.0),
                total_row: args.total_row,
                progress: args.progress,
                ratio: args.ratio,
                output: args.output,
                strict: args.strict,
                compare,
            })
        }
    }

    impl Config {
        fn convert_by_mode(mode: ByMode) -> ByKey {
            match mode { ByMode::Ext => ByKey::Ext, ByMode::Dir(d) => ByKey::Dir(d), ByMode::Mtime(g) => ByKey::Mtime(g), ByMode::None => unreachable!() }
        }
    }
}

// =====================================================================================
// Data Types
// =====================================================================================
mod types {
    use chrono::{DateTime, Local};
    use serde::Serialize;
    use std::path::PathBuf;

    #[derive(Debug, Clone)]
    pub struct FileMeta {
        pub size: u64,
        pub mtime: Option<DateTime<Local>>,
        pub is_text: bool,
        pub ext: String,
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct FileEntry { pub path: PathBuf, pub meta: FileMeta }

    #[derive(Debug, Clone)]
    pub struct FileStats {
        pub path: PathBuf,
        pub lines: usize,
        pub chars: usize,
        pub words: Option<usize>,
        pub size: u64,
        pub mtime: Option<DateTime<Local>>,
        pub ext: String,
        pub name: String,
    }

    impl FileStats {
        pub fn new(path: PathBuf, lines: usize, chars: usize, words: Option<usize>, meta: &FileMeta) -> Self {
            Self { path, lines, chars, words, size: meta.size, mtime: meta.mtime, ext: meta.ext.clone(), name: meta.name.clone() }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Summary { pub lines: usize, pub chars: usize, pub words: usize, pub files: usize }

    impl Summary {
        pub fn from_stats(stats: &[FileStats]) -> Self {
            let (lines, chars, words) = stats.iter().fold((0, 0, 0), |(l, c, w), s| (l + s.lines, c + s.chars, w + s.words.unwrap_or(0)));
            Self { lines, chars, words, files: stats.len() }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct JsonOutput { pub version: &'static str, pub files: Vec<JsonFile>, pub summary: JsonSummary, #[serde(skip_serializing_if = "Option::is_none")] pub by: Option<Vec<JsonGroup>> }

    #[derive(Debug, Serialize)]
    pub struct JsonFile { pub file: String, pub lines: usize, pub chars: usize, #[serde(skip_serializing_if = "Option::is_none")] pub words: Option<usize>, pub size: u64, #[serde(skip_serializing_if = "Option::is_none")] pub mtime: Option<String>, pub ext: String }

    #[derive(Debug, Serialize)]
    pub struct JsonSummary { pub lines: usize, pub chars: usize, #[serde(skip_serializing_if = "Option::is_none")] pub words: Option<usize>, pub files: usize }

    #[derive(Debug, Serialize)]
    pub struct JsonGroupRow { pub key: String, pub lines: usize, pub chars: usize, pub count: usize }

    #[derive(Debug, Serialize)]
    pub struct JsonGroup { pub label: String, pub rows: Vec<JsonGroupRow> }
}

// =====================================================================================
// Utilities
// =====================================================================================
mod util {
    use anyhow::Context as _;
    use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
    use std::path::{Path, PathBuf};

    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    pub fn parse_patterns(patterns: &[String]) -> anyhow::Result<Vec<glob::Pattern>> {
        patterns
            .iter()
            .map(|p| glob::Pattern::new(p).with_context(|| format!("Invalid pattern: {p}")))
            .collect()
    }

    #[derive(Debug, Clone, Copy)]
    pub struct SizeArg(pub u64);

    impl std::str::FromStr for SizeArg {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let s = s.trim().replace('_', "");
            let lower = s.to_ascii_lowercase();
            let (num_str, multiplier) = Self::parse_with_suffix(&lower)?;
            let num: u64 = num_str.parse().map_err(|_| format!("Invalid size number: {num_str}"))?;
            Ok(SizeArg(num * multiplier))
        }
    }

    impl SizeArg {
        fn parse_with_suffix(s: &str) -> Result<(&str, u64), String> {
            const SUFFIXES: &[(&[&str], u64)] = &[
                (&["tib", "tb", "t"], TB),
                (&["gib", "gb", "g"], GB),
                (&["mib", "mb", "m"], MB),
                (&["kib", "kb", "k"], KB),
            ];
            for (suffixes, multiplier) in SUFFIXES {
                for suffix in *suffixes {
                    if let Some(stripped) = s.strip_suffix(suffix) { return Ok((stripped.trim(), *multiplier)); }
                }
            }
            Ok((s, 1))
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DateTimeArg(pub DateTime<Local>);

    impl std::str::FromStr for DateTimeArg {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Self::try_rfc3339(s)
                .or_else(|| Self::try_datetime_format(s))
                .or_else(|| Self::try_date_format(s))
                .ok_or_else(|| format!("Cannot parse datetime: {s}"))
        }
    }

    impl DateTimeArg {
        fn try_rfc3339(s: &str) -> Option<Self> {
            chrono::DateTime::parse_from_rfc3339(s).ok().map(|dt| DateTimeArg(dt.with_timezone(&Local)))
        }
        fn try_datetime_format(s: &str) -> Option<Self> {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .ok()
                .and_then(|ndt| Local.from_local_datetime(&ndt).single())
                .map(DateTimeArg)
        }
        fn try_date_format(s: &str) -> Option<Self> {
            NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|nd| nd.and_hms_opt(0, 0, 0))
                .and_then(|ndt| Local.from_local_datetime(&ndt).single())
                .map(DateTimeArg)
        }
    }

    pub fn logical_absolute(path: &Path) -> PathBuf {
        if path.is_absolute() { path.to_path_buf() } else { std::env::current_dir().map(|cwd| cwd.join(path)).unwrap_or_else(|_| path.to_path_buf()) }
    }

    pub fn format_path(path: &Path, abs_path: bool, abs_canonical: bool, trim_root: Option<&Path>) -> String {
        let mut path = if abs_path {
            if abs_canonical { path.canonicalize().unwrap_or_else(|_| logical_absolute(path)) } else { logical_absolute(path) }
        } else {
            path.to_path_buf()
        };
        if let Some(root) = trim_root {
            if let Ok(stripped) = path.strip_prefix(root) { path = stripped.to_path_buf(); }
        }
        path.display().to_string()
    }

    pub fn get_dir_key(path: &Path, depth: usize) -> String {
        use std::path::Component;
        let base = path.parent().unwrap_or(Path::new("."));
        let parts: Vec<String> = base
            .components()
            .filter_map(|c| match c { Component::Normal(s) => Some(s.to_string_lossy().into_owned()), _ => None })
            .take(depth)
            .collect();
        if parts.is_empty() { ".".to_string() 
        } else { parts.join("/") }
    }

    pub fn mtime_bucket(dt: DateTime<Local>, g: crate::cli::Granularity) -> String {
        use chrono::Datelike;
        match g {
            crate::cli::Granularity::Day => dt.format("%Y-%m-%d").to_string(),
            crate::cli::Granularity::Week => format!("{:04}-W{:02}", dt.iso_week().year(), dt.iso_week().week()),
            crate::cli::Granularity::Month => dt.format("%Y-%m").to_string(),
        }
    }
}

// =====================================================================================
// File Collection
// =====================================================================================
mod files {
    use chrono::{DateTime, Local};
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read};
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    use crate::config::{Config, Filters};
    use crate::types::{FileEntry, FileMeta};

    const DEFAULT_PRUNE_DIRS: &[&str] = &[
        ".git", ".hg", ".svn", "node_modules", ".venv", "venv", "build", "dist", "target",
        ".cache", ".direnv", ".mypy_cache", ".pytest_cache", "coverage", "__pycache__",
        ".idea", ".next", ".nuxt",
    ];

    pub fn collect_entries(config: &Config) -> anyhow::Result<Vec<FileEntry>> {
        if let Some(ref from0) = config.files_from0 { return read_files_from_null(from0).map(|files| to_entries(files, config)); }
        if let Some(ref from) = config.files_from { return read_files_from_lines(from).map(|files| to_entries(files, config)); }
        if config.use_git {
            if let Ok(files) = collect_git_files(config) { return Ok(to_entries(files, config)); }
        }
        collect_walk_entries(config)
    }

    fn to_entries(files: Vec<PathBuf>, config: &Config) -> Vec<FileEntry> {
        files
            .into_iter()
            .filter_map(|p| FileMeta::from_path(&p, config).map(|meta| FileEntry { path: p, meta }))
            .collect()
    }

    fn read_files_from_lines(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(reader
            .lines()
            .filter_map(Result::ok)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }

    fn read_files_from_null(path: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut file = File::open(path)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf
            .split(|&b| b == 0)
            .filter_map(|chunk| {
                if chunk.is_empty() { return None; }
                let s = String::from_utf8_lossy(chunk);
                let trimmed = s.trim();
                (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
            })
            .collect())
    }

    fn collect_git_files(config: &Config) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for root in &config.paths {
            let output = std::process::Command::new("git")
                .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "."])
                .current_dir(root)
                .output()?;
            if !output.status.success() { anyhow::bail!("git ls-files failed"); }
            for chunk in output.stdout.split(|&b| b == 0) {
                if let Some(path_str) = parse_git_output_chunk(chunk) { files.push(root.join(path_str)); }
            }
        }
        files.sort();
        files.dedup();
        Ok(files)
    }

    fn parse_git_output_chunk(chunk: &[u8]) -> Option<String> {
        if chunk.is_empty() { return None; }
        let s = String::from_utf8_lossy(chunk).trim().to_string();
        (!s.is_empty()).then_some(s)
    }

    fn is_hidden(path: &Path) -> bool {
        path.file_name().map_or(false, |name| name.to_string_lossy().starts_with('.'))
    }

    fn should_process_entry(entry: &walkdir::DirEntry, config: &Config) -> bool {
        let path = entry.path();
        if !config.hidden && is_hidden(path) { return false; }
        if !config.no_default_prune && entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            if DEFAULT_PRUNE_DIRS.contains(&name.as_ref()) { return false; }
        }
        if entry.file_type().is_dir() { return !config.filters.exclude_dirs.iter().any(|p| p.matches_path(path)); }
        true
    }

    pub fn collect_walk_entries(config: &Config) -> anyhow::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        for root in &config.paths {
            let walker = WalkDir::new(root)
                .follow_links(config.follow)
                .into_iter()
                .filter_entry(|e| should_process_entry(e, config));

            for entry in walker.flatten() {
                if !entry.file_type().is_file() { continue; }
                let path = entry.path();
                if !PathMatcher::matches(path, config) { continue; }
                if let Some(meta) = FileMeta::from_path(path, config) {
                    entries.push(FileEntry { path: path.to_path_buf(), meta });
                }
            }
        }
        Ok(entries)
    }

    struct PathMatcher;

    impl PathMatcher {
        fn matches(path: &Path, config: &Config) -> bool {
            let filters = &config.filters;
            Self::matches_name(path, filters)
                && Self::matches_path_patterns(path, filters)
                && Self::matches_extension(path, filters)
                && Self::matches_metadata(path, config)
        }
        fn matches_name(path: &Path, filters: &Filters) -> bool {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !filters.include_patterns.is_empty() && !filters.include_patterns.iter().any(|p| p.matches(&name)) { return false; }
            !filters.exclude_patterns.iter().any(|p| p.matches(&name))
        }
        fn matches_path_patterns(path: &Path, filters: &Filters) -> bool {
            if !filters.include_paths.is_empty() && !filters.include_paths.iter().any(|p| p.matches_path(path)) { return false; }
            !filters.exclude_paths.iter().any(|p| p.matches_path(path))
        }
        fn matches_extension(path: &Path, filters: &Filters) -> bool {
            if filters.ext_filters.is_empty() { return true; }
            path.extension()
                .and_then(|e| Some(e.to_string_lossy().to_lowercase()))
                .map_or(false, |ext| filters.ext_filters.contains(&ext))
        }
        fn matches_metadata(path: &Path, config: &Config) -> bool {
            let filters = &config.filters;
            let metadata = match std::fs::metadata(path) { Ok(m) => m, Err(_) => return true };
            if !filters.size_range.contains(metadata.len()) { return false; }
            Self::matches_mtime(&metadata, config)
        }
        fn matches_mtime(metadata: &std::fs::Metadata, config: &Config) -> bool {
            let Ok(modified_sys) = metadata.modified() else { return true }; 
            let modified: DateTime<Local> = modified_sys.into();
            if let Some(since) = config.mtime_since { if modified < since { return false; } }
            if let Some(until) = config.mtime_until { if modified > until { return false; } }
            true
        }
    }

    impl FileMeta {
        pub fn from_path(path: &Path, config: &Config) -> Option<Self> {
            let metadata = std::fs::metadata(path).ok()?;
            let size = metadata.len();
            let mtime = metadata.modified().ok().map(Into::into);

            let is_text = if config.fast_text_detect { Self::quick_text_check(path) } else { Self::strict_text_check(path) };
            let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
            let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            Some(Self { size, mtime, is_text, ext, name })
        }
        fn quick_text_check(path: &Path) -> bool {
            let Ok(mut file) = File::open(path) else { return false }; 
            let mut buf = [0u8; 1024];
            let n = file.read(&mut buf).unwrap_or(0);
            !buf[..n].contains(&0)
        }
        fn strict_text_check(path: &Path) -> bool {
            let Ok(mut file) = File::open(path) else { return false }; 
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).is_ok() && !buf.contains(&0)
        }
    }
}

// =====================================================================================
// Compute
// =====================================================================================
mod compute {
    use rayon::prelude::*;
    use std::cmp::Ordering;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read};
    use std::path::Path;

    use crate::cli::SortKey;
    use crate::config::{ByKey, Config};
    use crate::types::{FileEntry, FileMeta, FileStats};
    use evalexpr::{ContextWithMutableVariables, Value};

    pub fn process_entries(config: &Config) -> anyhow::Result<Vec<FileStats>> {
        let entries = crate::files::collect_entries(config)?;
        let pool = rayon::ThreadPoolBuilder::new().num_threads(config.jobs).build()?;
        let stats = pool.install(|| entries.par_iter().filter_map(|e| FileMeasurer::measure(e, config)).collect());
        Ok(stats)
    }

    pub fn apply_sort(stats: &mut [FileStats], config: &Config) {
        if config.total_only || config.summary_only || config.sort_specs.is_empty() { return; }
        for (key, desc) in config.sort_specs.iter().rev() {
            stats.sort_by(|a, b| { let ord = Sorter::compare(a, b, *key); if *desc { ord.reverse() } else { ord } });
        }
    }

    struct Sorter;
    impl Sorter {
        fn compare(a: &FileStats, b: &FileStats, key: SortKey) -> Ordering {
            match key {
                SortKey::Lines => a.lines.cmp(&b.lines),
                SortKey::Chars => a.chars.cmp(&b.chars),
                SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
                SortKey::Name => a.path.cmp(&b.path),
                SortKey::Ext => a.ext.cmp(&b.ext),
            }
        }
    }

    struct FileMeasurer;
    impl FileMeasurer {
        fn measure(entry: &FileEntry, config: &Config) -> Option<FileStats> {
            if config.text_only && !entry.meta.is_text { return None; }
            let stats = if config.count_newlines_in_chars { Self::measure_whole(&entry.path, &entry.meta, config)? } else { Self::measure_by_lines(&entry.path, &entry.meta, config)? };
            Self::apply_filters(stats, config)
        }
        fn measure_whole(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
            let mut file = File::open(path).ok()?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok()?;
            if config.text_only && buf.contains(&0) { return None; }
            let content = String::from_utf8_lossy(&buf);
            let bytes = content.as_bytes();
            let newline_count = bytecount::count(bytes, b'\n');
            let lines = if bytes.is_empty() { 0 } else if bytes.last() == Some(&b'\n') { newline_count } else { newline_count + 1 };
            let chars = content.chars().count();
            let words = config.words.then(|| content.split_whitespace().count());
            Some(FileStats::new(path.to_path_buf(), lines, chars, words, meta))
        }
        fn measure_by_lines(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
            let file = File::open(path).ok()?;
            let mut reader = BufReader::new(file);
            let (mut lines, mut chars, mut words) = (0, 0, 0);
            let mut line = String::new();
            loop {
                line.clear();
                let n = reader.read_line(&mut line).ok()?;
                if n == 0 { break; }
                if line.ends_with('\n') { line.pop(); if line.ends_with('\r') { line.pop(); } }
                lines += 1;
                chars += line.chars().count();
                if config.words { words += line.split_whitespace().count(); }
            }
            Some(FileStats::new(path.to_path_buf(), lines, chars, config.words.then_some(words), meta))
        }
        fn apply_filters(stats: FileStats, config: &Config) -> Option<FileStats> {
            if !config.filters.lines_range.contains(stats.lines) { return None; }
            if !config.filters.chars_range.contains(stats.chars) { return None; }
            if !config.filters.words_range.contains(stats.words.unwrap_or(0)) { return None; }
            if let Some(ast) = &config.filters.filter_ast { if !Self::eval_filter(&stats, ast)? { return None; } }
            Some(stats)
        }
        fn eval_filter(stats: &FileStats, ast: &evalexpr::Node) -> Option<bool> {
            let mut ctx = evalexpr::HashMapContext::new();
            ctx.set_value("lines".into(), Value::Int(stats.lines as i64)).ok()?;
            ctx.set_value("chars".into(), Value::Int(stats.chars as i64)).ok()?;
            ctx.set_value("words".into(), Value::Int(stats.words.unwrap_or(0) as i64)).ok()?;
            ctx.set_value("size".into(), Value::Int(stats.size as i64)).ok()?;
            ctx.set_value("ext".into(), Value::String(stats.ext.clone())).ok()?;
            ctx.set_value("name".into(), Value::String(stats.name.clone())).ok()?;
            if let Some(mt) = stats.mtime { ctx.set_value("mtime".into(), Value::Int(mt.timestamp())).ok()?; }
            ast.eval_boolean_with_context(&ctx).ok()
        }
    }

    #[derive(Debug, Clone)]
    pub struct AggregationGroup { pub key: String, pub lines: usize, pub chars: usize, pub count: usize }

    impl AggregationGroup { fn new(key: String, lines: usize, chars: usize, count: usize) -> Self { Self { key, lines, chars, count } } }

    pub struct Aggregator;
    impl Aggregator {
        pub fn aggregate(stats: &[FileStats], by_keys: &[ByKey]) -> Vec<(String, Vec<AggregationGroup>)> {
            by_keys.iter().map(|key| Self::aggregate_by_key(stats, key)).collect()
        }
        fn aggregate_by_key(stats: &[FileStats], key: &ByKey) -> (String, Vec<AggregationGroup>) {
            match key { ByKey::Ext => Self::aggregate_by_ext(stats), ByKey::Dir(depth) => Self::aggregate_by_dir(stats, *depth), ByKey::Mtime(gran) => Self::aggregate_by_mtime(stats, *gran) }
        }
        fn aggregate_by_ext(stats: &[FileStats]) -> (String, Vec<AggregationGroup>) {
            let map = Self::build_aggregation_map(stats, |s| if s.ext.is_empty() { "(noext)".to_string() } else { s.ext.clone() });
            ("By Extension".to_string(), Self::map_to_sorted_groups(map))
        }
        fn aggregate_by_dir(stats: &[FileStats], depth: usize) -> (String, Vec<AggregationGroup>) {
            let map = Self::build_aggregation_map(stats, |s| crate::util::get_dir_key(&s.path, depth));
            (format!("By Directory (depth={depth})"), Self::map_to_sorted_groups(map))
        }
        fn aggregate_by_mtime(stats: &[FileStats], gran: crate::cli::Granularity) -> (String, Vec<AggregationGroup>) {
            let map = Self::build_aggregation_map(stats, |s| s.mtime.map(|mt| crate::util::mtime_bucket(mt, gran)).unwrap_or_else(|| "(no mtime)".to_string()));
            let gran_label = match gran { crate::cli::Granularity::Day => "day", crate::cli::Granularity::Week => "week", crate::cli::Granularity::Month => "month" };
            (format!("By Mtime ({gran_label})"), Self::map_to_sorted_groups(map))
        }
        fn build_aggregation_map<F>(stats: &[FileStats], key_fn: F) -> HashMap<String, (usize, usize, usize)> where F: Fn(&FileStats) -> String {
            let mut map: HashMap<String, (usize, usize, usize)> = HashMap::new();
            for stat in stats {
                let key = key_fn(stat);
                let entry = map.entry(key).or_insert((0, 0, 0));
                entry.0 += stat.lines;
                entry.1 += stat.chars;
                entry.2 += 1;
            }
            map
        }
        fn map_to_sorted_groups(map: HashMap<String, (usize, usize, usize)>) -> Vec<AggregationGroup> {
            let mut groups: Vec<AggregationGroup> = map.into_iter().map(|(key, (lines, chars, count))| AggregationGroup::new(key, lines, chars, count)).collect();
            groups.sort_by(|a, b| b.lines.cmp(&a.lines));
            groups
        }
    }
}

// =====================================================================================
// Output
// =====================================================================================
mod output {
    use std::io::Write;

    use crate::config::Config;
    use crate::types::{FileStats, JsonFile, JsonGroup, JsonGroupRow, JsonOutput, JsonSummary, Summary};

    pub fn emit(stats: &[FileStats], config: &Config) -> anyhow::Result<()> {
        let mut writer = OutputWriter::create(config)?;
        match config.format {
            crate::cli::OutputFormat::Json => output_json(stats, config, &mut writer)?,
            crate::cli::OutputFormat::Yaml => output_yaml(stats, config, &mut writer)?,
            crate::cli::OutputFormat::Csv => output_delimited(stats, config, ',', &mut writer)?,
            crate::cli::OutputFormat::Tsv => output_delimited(stats, config, '\t', &mut writer)?,
            crate::cli::OutputFormat::Table => output_table(stats, config, &mut writer)?,
            crate::cli::OutputFormat::Md => output_markdown(stats, config, &mut writer)?,
            crate::cli::OutputFormat::Jsonl => output_jsonl(stats, config, &mut writer)?,
        }
        Ok(())
    }

    struct OutputWriter(Box<dyn Write>);
    impl OutputWriter {
        fn create(config: &Config) -> anyhow::Result<Self> {
            let writer: Box<dyn Write> = if let Some(path) = &config.output { Box::new(std::io::BufWriter::new(std::fs::File::create(path)?)) } else { Box::new(std::io::BufWriter::new(std::io::stdout())) };
            Ok(Self(writer))
        }
    }
    impl Write for OutputWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.0.write(buf) }
        fn flush(&mut self) -> std::io::Result<()> { self.0.flush() }
    }

    fn limited<'a>(stats: &'a [FileStats], config: &Config) -> &'a [FileStats] {
        let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
        &stats[..limit]
    }

    fn format_ratio(val: usize, total: usize) -> String {
        if total == 0 { "0.0".into() } else { format!("{:.1}", (val as f64) * 100.0 / (total as f64)) }
    }

    fn format_path(stats: &FileStats, config: &Config) -> String {
        crate::util::format_path(&stats.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref())
    }

    fn output_table(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        if config.total_only { return output_summary(stats, config, out); }
        if !config.summary_only { write_table_header(config, out)?; write_table_rows(stats, config, out)?; }
        if !config.total_only { write_aggregations(stats, config, out)?; }
        output_summary(stats, config, out)
    }

    fn write_table_header(config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        writeln!(out)?;
        if config.words {
            if config.ratio { writeln!(out, "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\t   WORDS\tFILE")?; }
            else { writeln!(out, "    LINES\t CHARACTERS\t   WORDS\tFILE")?; }
        } else if config.ratio { writeln!(out, "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\tFILE")?; }
        else { writeln!(out, "    LINES\t CHARACTERS\tFILE")?; }
        writeln!(out, "----------------------------------------------")?;
        Ok(())
    }

    fn write_table_rows(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let summary = Summary::from_stats(stats);
        for s in limited(stats, config) {
            let path = format_path(s, config);
            if config.words {
                if config.ratio {
                    writeln!(out, "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{:>7}\t{}", format_ratio(s.lines, summary.lines), s.lines, format_ratio(s.chars, summary.chars), s.chars, s.words.unwrap_or(0), path)?;
                } else {
                    writeln!(out, "{:>10}\t{:>10}\t{:>7}\t{}", s.lines, s.chars, s.words.unwrap_or(0), path)?;
                }
            } else if config.ratio {
                writeln!(out, "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{}", format_ratio(s.lines, summary.lines), s.lines, format_ratio(s.chars, summary.chars), s.chars, path)?;
            } else {
                writeln!(out, "{:>10}\t{:>10}\t{}", s.lines, s.chars, path)?;
            }
        }
        writeln!(out, "---")?;
        Ok(())
    }

    fn write_aggregations(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let groups = crate::compute::Aggregator::aggregate(stats, &config.by_modes);
        for (label, mut rows) in groups {
            writeln!(out, "[{label}]")?;
            writeln!(out, "{:>10}\t{:>10}\tKEY", "LINES", "CHARACTERS")?;
            if let Some(n) = config.by_limit { rows.truncate(n); }
            for g in rows { writeln!(out, "{:>10}\t{:>10}\t{} ({} files)", g.lines, g.chars, g.key, g.count)?; }
            writeln!(out, "---")?;
        }
        Ok(())
    }

    fn output_summary(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let summary = Summary::from_stats(stats);
        if config.words { writeln!(out, "{:>10}\t{:>10}\t{:>7}\tTOTAL ({} files)\n", summary.lines, summary.chars, summary.words, summary.files)?; }
        else { writeln!(out, "{:>10}\t{:>10}\tTOTAL ({} files)\n", summary.lines, summary.chars, summary.files)?; }
        Ok(())
    }

    fn output_delimited(stats: &[FileStats], config: &Config, sep: char, out: &mut impl Write) -> anyhow::Result<()> {
        write_delimited_header(config, sep, out)?;
        write_delimited_rows(stats, config, sep, out)?;
        if config.total_row { write_delimited_total(stats, config, sep, out)?; }
        Ok(())
    }

    fn write_delimited_header(config: &Config, sep: char, out: &mut impl Write) -> anyhow::Result<()> {
        if config.words { writeln!(out, "lines{sep}chars{sep}words{sep}file")?; }
        else { writeln!(out, "lines{sep}chars{sep}file")?; }
        Ok(())
    }

    fn write_delimited_rows(stats: &[FileStats], config: &Config, sep: char, out: &mut impl Write) -> anyhow::Result<()> {
        for s in limited(stats, config) {
            let path = escape_field(&format_path(s, config), sep);
            if config.words { writeln!(out, "{}{sep}{}{sep}{}{sep}{}", s.lines, s.chars, s.words.unwrap_or(0), path)?; }
            else { writeln!(out, "{}{sep}{}{sep}{}", s.lines, s.chars, path)?; }
        }
        Ok(())
    }

    fn write_delimited_total(stats: &[FileStats], config: &Config, sep: char, out: &mut impl Write) -> anyhow::Result<()> {
        let summary = Summary::from_stats(stats);
        let total_label = escape_field("TOTAL", sep);
        if config.words { writeln!(out, "{}{sep}{}{sep}{}{sep}{}", summary.lines, summary.chars, summary.words, total_label)?; }
        else { writeln!(out, "{}{sep}{}{sep}{}", summary.lines, summary.chars, total_label)?; }
        Ok(())
    }

    fn escape_field(s: &str, sep: char) -> String {
        if sep == ',' { let escaped = s.replace('"', "\"\""); format!("\"{escaped}\"") } else { s.to_string() }
    }

    fn output_markdown(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        write_markdown_header(config, out)?;
        write_markdown_rows(stats, config, out)?;
        write_markdown_aggregations(stats, config, out)?;
        Ok(())
    }

    fn write_markdown_header(config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        if config.words {
            if config.ratio { writeln!(out, "| LINES% | LINES | CHARS% | CHARS | WORDS | FILE |\n|---:|---:|---:|---:|---:|:---|")?; }
            else { writeln!(out, "| LINES | CHARS | WORDS | FILE |\n|---:|---:|---:|:---|")?; }
        } else if config.ratio { writeln!(out, "| LINES% | LINES | CHARS% | CHARS | FILE |\n|---:|---:|---:|---:|:---|")?; }
        else { writeln!(out, "| LINES | CHARS | FILE |\n|---:|---:|:---|")?; }
        Ok(())
    }

    fn write_markdown_rows(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let summary = Summary::from_stats(stats);
        for s in limited(stats, config) {
            let path = format_path(s, config).replace('|', "\\|");
            if config.words {
                if config.ratio {
                    writeln!(out, "| {} | {} | {} | {} | {} | {} |", format_ratio(s.lines, summary.lines), s.lines, format_ratio(s.chars, summary.chars), s.chars, s.words.unwrap_or(0), path)?;
                } else {
                    writeln!(out, "| {} | {} | {} | {} |", s.lines, s.chars, s.words.unwrap_or(0), path)?;
                }
            } else if config.ratio {
                writeln!(out, "| {} | {} | {} | {} | {} |", format_ratio(s.lines, summary.lines), s.lines, format_ratio(s.chars, summary.chars), s.chars, path)?;
            } else {
                writeln!(out, "| {} | {} | {} |", s.lines, s.chars, path)?;
            }
        }
        Ok(())
    }

    fn write_markdown_aggregations(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let groups = crate::compute::Aggregator::aggregate(stats, &config.by_modes);
        for (label, mut rows) in groups {
            writeln!(out, "\n### {label}\n")?;
            writeln!(out, "| LINES | CHARS | KEY | COUNT |\n|---:|---:|:---|---:|")?;
            if let Some(n) = config.by_limit { rows.truncate(n); }
            for g in rows { let key = g.key.replace('|', "\\|"); writeln!(out, "| {} | {} | {} | {} |", g.lines, g.chars, key, g.count)?; }
        }
        Ok(())
    }

    fn build_json_output(stats: &[FileStats], config: &Config) -> JsonOutput {
        let files = stats.iter().map(|s| JsonFile { file: format_path(s, config), lines: s.lines, chars: s.chars, words: s.words, size: s.size, mtime: s.mtime.map(|d| d.to_rfc3339()), ext: s.ext.clone() }).collect();
        let summary_data = Summary::from_stats(stats);
        let summary = JsonSummary { lines: summary_data.lines, chars: summary_data.chars, words: config.words.then_some(summary_data.words), files: summary_data.files };
        let by = build_json_groups(stats, config);
        JsonOutput { version: crate::VERSION, files, summary, by }
    }

    fn build_json_groups(stats: &[FileStats], config: &Config) -> Option<Vec<JsonGroup>> {
        let groups = crate::compute::Aggregator::aggregate(stats, &config.by_modes);
        if groups.is_empty() { return None; }
        let json_groups = groups
            .into_iter()
            .map(|(label, mut rows)| {
                if let Some(n) = config.by_limit { rows.truncate(n); }
                let json_rows = rows.into_iter().map(|g| JsonGroupRow { key: g.key, lines: g.lines, chars: g.chars, count: g.count }).collect();
                JsonGroup { label, rows: json_rows }
            })
            .collect();
        Some(json_groups)
    }

    fn output_json(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let output = build_json_output(stats, config);
        serde_json::to_writer_pretty(&mut *out, &output)?;
        writeln!(out)?;
        Ok(())
    }

    fn output_yaml(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        let output = build_json_output(stats, config);
        let yaml_str = serde_yaml::to_string(&output)?;
        writeln!(out, "{}", yaml_str)?;
        Ok(())
    }

    fn output_jsonl(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
        for s in stats {
            let item = serde_json::json!({
                "type": "file",
                "file": format_path(s, config),
                "lines": s.lines,
                "chars": s.chars,
                "words": s.words,
                "size": s.size,
                "mtime": s.mtime.map(|d| d.to_rfc3339()),
                "ext": &s.ext,
            });
            serde_json::to_writer(&mut *out, &item)?;
            writeln!(out)?;
        }
        let summary = Summary::from_stats(stats);
        let total = serde_json::json!({
            "type": "total",
            "version": crate::VERSION,
            "lines": summary.lines,
            "chars": summary.chars,
            "words": if config.words { Some(summary.words) } else { None },
            "files": summary.files,
        });
        serde_json::to_writer(&mut *out, &total)?;
        writeln!(out)?;
        Ok(())
    }
}

// =====================================================================================
// Compare
// =====================================================================================
mod compare {
    use anyhow::Result;
    use serde::Deserialize;
    use std::collections::HashMap;
    use std::path::Path;

    #[derive(Debug, Deserialize)]
    struct FileSummary { lines: usize, chars: usize, words: Option<usize>, files: usize }

    #[derive(Debug, Deserialize)]
    struct FileItem { file: String, lines: usize, chars: usize, words: Option<usize> }

    #[derive(Debug, Deserialize)]
    struct Snapshot { files: Vec<FileItem>, summary: FileSummary }

    pub fn run(old_path: &Path, new_path: &Path) -> Result<String> {
        let old: Snapshot = serde_json::from_reader(std::fs::File::open(old_path)?)?;
        let new: Snapshot = serde_json::from_reader(std::fs::File::open(new_path)?)?;
        let comparison = SnapshotComparison::new(old, new);
        Ok(comparison.format())
    }

    struct SnapshotComparison { old: Snapshot, new: Snapshot }

    impl SnapshotComparison {
        fn new(old: Snapshot, new: Snapshot) -> Self { Self { old, new } }
        fn format(&self) -> String {
            let mut output = String::new();
            output.push_str("DIFF (new - old)\n");
            output.push_str(&self.format_summary_diff());
            output.push_str("\n[Changed files]\n");
            output.push_str(&self.format_file_diffs());
            output
        }
        fn format_summary_diff(&self) -> String {
            let mut output = String::new();
            output.push_str(&format!("Lines: {} -> {} (Δ {})\n", self.old.summary.lines, self.new.summary.lines, self.calculate_diff(self.new.summary.lines, self.old.summary.lines)));
            output.push_str(&format!("Chars: {} -> {} (Δ {})\n", self.old.summary.chars, self.new.summary.chars, self.calculate_diff(self.new.summary.chars, self.old.summary.chars)));
            output.push_str(&format!("Files: {} -> {} (Δ {})\n", self.old.summary.files, self.new.summary.files, self.calculate_diff(self.new.summary.files, self.old.summary.files)));
            if let (Some(ow), Some(nw)) = (self.old.summary.words, self.new.summary.words) {
                output.push_str(&format!("Words: {} -> {} (Δ {})\n", ow, nw, self.calculate_diff(nw, ow)));
            }
            output
        }
        fn format_file_diffs(&self) -> String {
            let old_map: HashMap<&str, &FileItem> = self.old.files.iter().map(|f| (f.file.as_str(), f)).collect();
            let mut output = String::new();
            for new_file in &self.new.files {
                if let Some(old_file) = old_map.get(new_file.file.as_str()) {
                    let lines_diff = self.calculate_diff(new_file.lines, old_file.lines);
                    let chars_diff = self.calculate_diff(new_file.chars, old_file.chars);
                    let words_diff = match (old_file.words, new_file.words) { (Some(ow), Some(nw)) => Some(self.calculate_diff(nw, ow)), _ => None };
                    if lines_diff != 0 || chars_diff != 0 || words_diff.unwrap_or(0) != 0 {
                        if let Some(wd) = words_diff { output.push_str(&format!("{:>10} L  {:>10} C  {:>10} W  {}\n", lines_diff, chars_diff, wd, new_file.file)); }
                        else { output.push_str(&format!("{:>10} L  {:>10} C  {}\n", lines_diff, chars_diff, new_file.file)); }
                    }
                } else {
                    output.push_str(&format!("{:>10} L  {:>10} C  {} (added)\n", new_file.lines as isize, new_file.chars as isize, new_file.file));
                }
            }
            output
        }
        fn calculate_diff(&self, new: usize, old: usize) -> isize { new as isize - old as isize }
    }
}

// =====================================================================================
// Main
// =====================================================================================
fn main() -> Result<()> {
    let args = cli::Args::parse();
    let config = config::Config::try_from(args)?;

    if let Some((old, new)) = &config.compare {
        let diff = compare::run(old, new).context("compare failed")?;
        println!("{}", diff);
        return Ok(());
    }

    if !matches!(config.format, cli::OutputFormat::Json) && atty::is(Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", VERSION, config.jobs);
    }

    if config.progress {
        eprintln!("[count_lines] scanning & measuring...");
    }

    let mut stats = match compute::process_entries(&config) {
        Ok(v) => v,
        Err(e) => {
            if config.strict { return Err(e).context("failed to measure entries"); }
            eprintln!("[warn] {}", e);
            Vec::new()
        }
    };

    compute::apply_sort(&mut stats, &config);
    output::emit(&stats, &config).context("failed to emit output")?;
    Ok(())
}
