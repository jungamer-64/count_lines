// src/main.rs
#![allow(clippy::multiple_crate_versions)]

use anyhow::{Context, Result};
use atty::Stream;
use clap::Parser;

const VERSION: &str = "2.4.0";

// -------------------------------------------------------------------------------------
// CLI
// -------------------------------------------------------------------------------------
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

    /// Aggregation key for summary output.
    #[derive(Debug, Clone, Copy)]
    pub enum ByMode {
        None,
        Ext,
        Dir(usize),
        Mtime(Granularity),
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Granularity { Day, Week, Month }

    impl std::str::FromStr for ByMode {
        type Err = String;
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            match s {
                "ext" => Ok(Self::Ext),
                x if x.starts_with("dir") => {
                    let depth = x
                        .strip_prefix("dir=")
                        .and_then(|d| d.parse().ok())
                        .unwrap_or(1);
                    Ok(Self::Dir(depth))
                }
                x if x.starts_with("mtime") => {
                    // mtime[:day|week|month]
                    let gran = x.split(':').nth(1).unwrap_or("day");
                    let g = match gran {
                        "day" => Granularity::Day,
                        "week" => Granularity::Week,
                        "month" => Granularity::Month,
                        _ => return Err(format!("Unknown mtime granularity: {gran}")),
                    };
                    Ok(Self::Mtime(g))
                }
                "none" => Ok(Self::None),
                other => Err(format!("Unknown --by mode: {other}")),
            }
        }
    }

    /// `--sort` 複数キー/方向の指定を表現
    #[derive(Debug, Clone)]
    pub struct SortSpec(pub Vec<(SortKey, bool)>);

    impl std::str::FromStr for SortSpec {
        type Err = String;
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            let mut v = Vec::new();
            for part in s.split(',').map(|p| p.trim()).filter(|p| !p.is_empty()) {
                let (key_str, desc) = if let Some((k, d)) = part.split_once(':') {
                    (k.trim(), matches!(d.trim(), "desc" | "DESC"))
                } else {
                    (part, false)
                };
                let key = match key_str.to_ascii_lowercase().as_str() {
                    "lines" => SortKey::Lines,
                    "chars" => SortKey::Chars,
                    "words" => SortKey::Words,
                    "name" => SortKey::Name,
                    "ext" => SortKey::Ext,
                    other => return Err(format!("Unknown sort key: {other}")),
                };
                v.push((key, desc));
            }
            if v.is_empty() { return Err("empty sort spec".into()); }
            Ok(SortSpec(v))
        }
    }

    #[derive(Parser, Debug)]
    #[command(
        name = "count_lines",
        version = crate::VERSION,
        about = "ファイル行数/文字数/単語数の集計ツール",
        long_about = Some(
            include_str!("../usage.txt")
        )
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

        /// 最小ファイルサイズ
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
        pub compare: Option<Vec<PathBuf>>, // [old, new]

        /// 対象パス
        pub paths: Vec<PathBuf>,
    }
}

// -------------------------------------------------------------------------------------
// App config + filters
// -------------------------------------------------------------------------------------
mod config {
    use super::cli::{ByMode, OutputFormat, SortKey};
    use crate::util::logical_absolute;
    use anyhow::Result;
    use chrono::{DateTime, Local};
    use evalexpr::Node;
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[derive(Debug, Default)]
    #[allow(clippy::struct_field_names)]
    pub struct Filters {
        pub include_patterns: Vec<glob::Pattern>,
        pub exclude_patterns: Vec<glob::Pattern>,
        pub include_paths: Vec<glob::Pattern>,
        pub exclude_paths: Vec<glob::Pattern>,
        pub exclude_dirs: Vec<glob::Pattern>,
        pub ext_filters: HashSet<String>,
        pub max_size: Option<u64>,
        pub min_size: Option<u64>,
        pub min_lines: Option<usize>,
        pub max_lines: Option<usize>,
        pub min_chars: Option<usize>,
        pub max_chars: Option<usize>,
        pub min_words: Option<usize>,
        pub max_words: Option<usize>,
        pub filter_ast: Option<Node>,
    }

    impl Filters {
        pub fn from_args(args: &crate::cli::Args) -> Result<Self> {
            let filter_ast = if let Some(expr) = &args.filter { Some(evalexpr::build_operator_tree(expr).map_err(|e| anyhow::anyhow!(e))?) } else { None };
            Ok(Self {
                include_patterns: crate::util::parse_patterns(&args.include)?,
                exclude_patterns: crate::util::parse_patterns(&args.exclude)?,
                include_paths: crate::util::parse_patterns(&args.include_path)?,
                exclude_paths: crate::util::parse_patterns(&args.exclude_path)?,
                exclude_dirs: crate::util::parse_patterns(&args.exclude_dir)?,
                ext_filters: args
                    .ext
                    .as_ref()
                    .map(|s| s.split(',').map(|e| e.trim().to_lowercase()).collect())
                    .unwrap_or_default(),
                max_size: args.max_size.map(|s| s.0),
                min_size: args.min_size.map(|s| s.0),
                min_lines: args.min_lines,
                max_lines: args.max_lines,
                min_chars: args.min_chars,
                max_chars: args.max_chars,
                min_words: args.min_words,
                max_words: args.max_words,
                filter_ast,
            })
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
            let jobs = args.jobs.unwrap_or_else(num_cpus::get);
            let paths = if args.paths.is_empty() { vec![PathBuf::from(".")] } else { args.paths };

            let by_modes: Vec<ByKey> = if args.by.is_empty() {
                vec![]
            } else {
                args.by
                    .into_iter()
                    .map(|b| match b {
                        ByMode::Ext => ByKey::Ext,
                        ByMode::Dir(d) => ByKey::Dir(d),
                        ByMode::Mtime(g) => ByKey::Mtime(g),
                        ByMode::None => unreachable!(),
                    })
                    .collect()
            };

            let compare = args.compare.and_then(|v| if v.len()==2 { Some((v[0].clone(), v[1].clone())) } else { None });

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

    // 公開 re-exports
    pub use Config as AppConfig;
    pub use Filters as AppFilters;
    pub use super::cli::Granularity;
}

// -------------------------------------------------------------------------------------
// Data types
// -------------------------------------------------------------------------------------
mod types {
    use serde::Serialize;
    use std::path::PathBuf;
    use chrono::{DateTime, Local};

    #[derive(Debug, Clone)]
    pub struct FileMeta {
        pub size: u64,
        pub mtime: Option<DateTime<Local>>,
        pub is_text: bool,
        pub ext: String,
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct FileEntry {
        pub path: PathBuf,
        pub meta: FileMeta,
    }

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

    #[derive(Debug, Serialize)]
    pub struct JsonOutput {
        pub files: Vec<JsonFile>,
        pub summary: JsonSummary,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub by: Option<Vec<JsonGroup>>, // 複合 by を配列で
    }

    #[derive(Debug, Serialize)]
    pub struct JsonFile {
        pub file: String,
        pub lines: usize,
        pub chars: usize,
        #[serde(skip_serializing_if = "Option::is_none")] pub words: Option<usize>,
        pub size: u64,
        #[serde(skip_serializing_if = "Option::is_none")] pub mtime: Option<String>,
        pub ext: String,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonSummary {
        pub lines: usize,
        pub chars: usize,
        #[serde(skip_serializing_if = "Option::is_none")] pub words: Option<usize>,
        pub files: usize,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonGroupRow { pub key: String, pub lines: usize, pub chars: usize, pub count: usize }

    #[derive(Debug, Serialize)]
    pub struct JsonGroup { pub label: String, pub rows: Vec<JsonGroupRow> }

    pub use FileEntry as Entry;
    pub use FileMeta as Meta;
    pub use FileStats as Stats;
    pub use JsonFile as OutFile;
    pub use JsonOutput as Out;
    pub use JsonSummary as OutSummary;
    pub use JsonGroup as OutGroup;
    pub use JsonGroupRow as OutGroupRow;
}

// -------------------------------------------------------------------------------------
// Utilities
// -------------------------------------------------------------------------------------
mod util {
    use anyhow::Context as _;
    use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
    use std::path::{Path, PathBuf};

    const KB: u64 = 1024; const MB: u64 = 1024 * KB; const GB: u64 = 1024 * MB; const TB: u64 = 1024 * GB;

    #[inline]
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
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            let s = s.trim().replace('_', "");
            let lower = s.to_ascii_lowercase();
            let parse_with_suffix = |suffixes: &[&str], multiplier: u64| {
                for suffix in suffixes { if let Some(stripped) = lower.strip_suffix(suffix) { return Some((stripped.trim(), multiplier)); } }
                None
            };
            let (num_str, mul) = parse_with_suffix(&["kib","kb","k"], KB)
                .or_else(|| parse_with_suffix(&["mib","mb","m"], MB))
                .or_else(|| parse_with_suffix(&["gib","gb","g"], GB))
                .or_else(|| parse_with_suffix(&["tib","tb","t"], TB))
                .unwrap_or((lower.as_str(), 1));
            let num: u64 = num_str.parse().map_err(|_| format!("Invalid size number: {num_str}"))?;
            Ok(SizeArg(num * mul))
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct DateTimeArg(pub DateTime<Local>);
    impl std::str::FromStr for DateTimeArg {
        type Err = String;
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) { return Ok(DateTimeArg(dt.with_timezone(&Local))); }
            if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                return Local.from_local_datetime(&ndt).single().ok_or_else(|| "Ambiguous datetime".to_string()).map(DateTimeArg);
            }
            if let Ok(nd) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                let ndt = nd.and_hms_opt(0,0,0).ok_or_else(|| "Invalid time".to_string())?;
                return Local.from_local_datetime(&ndt).single().ok_or_else(|| "Ambiguous datetime".to_string()).map(DateTimeArg);
            }
            Err(format!("Cannot parse datetime: {s}"))
        }
    }

    #[inline]
    pub fn logical_absolute(path: &Path) -> PathBuf {
        if path.is_absolute() { return path.to_path_buf(); }
        std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
    }

    pub fn format_path(path: &Path, abs_path: bool, abs_canonical: bool, trim_root: Option<&Path>) -> String {
        let mut path = if abs_path {
            if abs_canonical { path.canonicalize().unwrap_or_else(|_| logical_absolute(path)) } else { logical_absolute(path) }
        } else { path.to_path_buf() };
        if let Some(root) = trim_root { if let Ok(stripped) = path.strip_prefix(root) { path = stripped.to_path_buf(); } }
        path.display().to_string()
    }

    pub fn get_dir_key(path: &Path, depth: usize) -> String {
        use std::path::Component;
        let base = path.parent().unwrap_or(path);
        let mut parts = Vec::with_capacity(depth);
        for comp in base.components() {
            if let Component::Normal(s) = comp {
                parts.push(s.to_string_lossy().into_owned());
                if parts.len() >= depth { break; }
            }
        }
        if parts.is_empty() { ".".to_string() } else { parts.join("/") }
    }

    pub fn mtime_bucket(dt: chrono::DateTime<Local>, g: crate::cli::Granularity) -> String {
        use chrono::Datelike;
        match g {
            crate::cli::Granularity::Day => dt.format("%Y-%m-%d").to_string(),
            crate::cli::Granularity::Week => format!("{:04}-W{:02}", dt.iso_week().year(), dt.iso_week().week()),
            crate::cli::Granularity::Month => dt.format("%Y-%m").to_string(),
        }
    }
}

// -------------------------------------------------------------------------------------
// File collection
// -------------------------------------------------------------------------------------
mod files {
    use chrono::{DateTime, Local};
    use std::path::PathBuf;
    use walkdir::WalkDir;

    use crate::config::AppConfig;
    use crate::types::{Entry, Meta};

    const DEFAULT_PRUNE_DIRS: &[&str] = &[
        ".git", ".hg", ".svn", "node_modules", ".venv", "venv", "build", "dist", "target", ".cache", ".direnv", ".mypy_cache", ".pytest_cache", "coverage", "__pycache__", ".idea", ".next", ".nuxt",
    ];

    pub fn collect_entries(config: &AppConfig) -> anyhow::Result<Vec<Entry>> {
        if let Some(ref from0) = config.files_from0 { return read_files_from0(from0).map(|v| to_entries(v, config)); }
        if let Some(ref from) = config.files_from { return read_files_from(from).map(|v| to_entries(v, config)); }
        if config.use_git { if let Ok(files) = collect_git_files(config) { return Ok(to_entries(files, config)); } }
        collect_find_entries(config)
    }

    fn to_entries(files: Vec<PathBuf>, config: &AppConfig) -> Vec<Entry> {
        files.into_iter().filter_map(|p| meta_for(&p, config).map(|m| Entry { path: p, meta: m })).collect()
    }

    fn read_files_from(path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
        use std::fs::File; use std::io::{BufRead, BufReader};
        let file = File::open(path)?; let reader = BufReader::new(file);
        Ok(reader.lines().filter_map(Result::ok).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).map(PathBuf::from).collect())
    }

    fn read_files_from0(path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
        use std::fs::File; use std::io::Read;
        let mut f = File::open(path)?; let mut buf = Vec::new(); f.read_to_end(&mut buf)?;
        let mut out = Vec::new();
        for chunk in buf.split(|b| *b == 0) { if chunk.is_empty() { continue; } let s = String::from_utf8_lossy(chunk); let trimmed = s.trim(); if !trimmed.is_empty() { out.push(PathBuf::from(trimmed)); } }
        Ok(out)
    }

    fn collect_git_files(config: &AppConfig) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for root in &config.paths {
            let output = std::process::Command::new("git")
                .arg("ls-files").arg("-z").arg("--cached").arg("--others").arg("--exclude-standard").arg("--").arg(".")
                .current_dir(root)
                .output()?;
            if !output.status.success() { anyhow::bail!("git ls-files failed"); }
            for chunk in output.stdout.split(|b| *b == 0) {
                if chunk.is_empty() { continue; }
                let s = String::from_utf8_lossy(chunk).trim().to_string(); if s.is_empty() { continue; }
                files.push(root.join(s));
            }
        }
        files.sort(); files.dedup(); Ok(files)
    }

    #[inline]
    fn is_hidden(path: &std::path::Path) -> bool { path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('.')) }

    #[inline]
    fn should_process_entry(entry: &walkdir::DirEntry, config: &AppConfig) -> bool {
        let path = entry.path();
        if !config.hidden && is_hidden(path) { return false; }
        if !config.no_default_prune && entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy(); if DEFAULT_PRUNE_DIRS.contains(&name.as_ref()) { return false; }
        }
        if entry.file_type().is_dir() {
            for pattern in &config.filters.exclude_dirs { if pattern.matches_path(path) { return false; } }
        }
        true
    }

    fn matches_filters(path: &std::path::Path, config: &AppConfig) -> bool {
        use crate::config::AppFilters;
        let AppFilters { include_patterns, exclude_patterns, include_paths, exclude_paths, ext_filters, max_size, min_size, .. } = &config.filters;

        if !include_patterns.is_empty() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !include_patterns.iter().any(|p| p.matches(&name)) { return false; }
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if exclude_patterns.iter().any(|p| p.matches(&name)) { return false; }
        if !include_paths.is_empty() && !include_paths.iter().any(|p| p.matches_path(path)) { return false; }
        if exclude_paths.iter().any(|p| p.matches_path(path)) { return false; }
        if !ext_filters.is_empty() {
            match path.extension().map(|e| e.to_string_lossy().to_lowercase()) {
                Some(ext) if ext_filters.contains(&ext) => {}
                _ => return false,
            }
        }
        if max_size.is_some() || min_size.is_some() || config.mtime_since.is_some() || config.mtime_until.is_some() {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Some(max) = max_size { if metadata.len() > *max { return false; } }
                if let Some(min) = min_size { if metadata.len() < *min { return false; } }
                if let Ok(modified) = metadata.modified() {
                    let modified: DateTime<Local> = modified.into();
                    if let Some(since) = &config.mtime_since { if modified < *since { return false; } }
                    if let Some(until) = &config.mtime_until { if modified > *until { return false; } }
                }
            }
        }
        true
    }

    fn meta_for(path: &std::path::Path, config: &AppConfig) -> Option<Meta> {
        let md = std::fs::metadata(path).ok()?;
        let size = md.len();
        let mtime = md.modified().ok().map(|t| { let dt: DateTime<Local> = t.into(); dt });
        // テキスト判定
        let is_text = if config.fast_text_detect { quick_text_check(path) } else { strict_text_check(path) };
        let ext = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        Some(Meta { size, mtime, is_text, ext, name })
    }

    fn quick_text_check(path: &std::path::Path) -> bool {
        use std::fs::File; use std::io::Read; let Ok(mut f) = File::open(path) else { return false; };
        let mut buf = [0u8; 1024]; let n = f.read(&mut buf).unwrap_or(0); !buf[..n].contains(&0)
    }
    fn strict_text_check(path: &std::path::Path) -> bool {
        use std::fs::File;
        use std::io::Read;
        let Ok(mut f) = File::open(path) else { return false; };
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() { return false; }
        !buf.contains(&0)
        }

    pub fn collect_find_entries(config: &AppConfig) -> anyhow::Result<Vec<Entry>> {
        let mut entries = Vec::new();
        for root in &config.paths {
            let walker = WalkDir::new(root).follow_links(config.follow).into_iter().filter_entry(|e| should_process_entry(e, config));
            for entry in walker {
                let Ok(entry) = entry else { continue };
                if !entry.file_type().is_file() { continue; }
                let path = entry.path();
                if !matches_filters(path, config) { continue; }
                if let Some(meta) = meta_for(path, config) { entries.push(Entry { path: path.to_path_buf(), meta }); }
            }
        }
        Ok(entries)
    }
}

// -------------------------------------------------------------------------------------
// Compute
// -------------------------------------------------------------------------------------
mod compute {
    use rayon::prelude::*;

    use crate::cli::SortKey;
    use crate::config::{AppConfig, ByKey};
    use crate::types::{Entry, Stats as FileStats};
    use evalexpr::{ContextWithMutableVariables, Value};

    pub const READ_BUF_BYTES: usize = 8192;

    pub fn process_entries(config: &AppConfig) -> anyhow::Result<Vec<FileStats>> {
        let entries = crate::files::collect_entries(config)?;
        let pool = rayon::ThreadPoolBuilder::new().num_threads(config.jobs).build()?;

        let stats: Vec<FileStats> = pool.install(|| {
            entries.par_iter().filter_map(|e| measure(e, config)).collect()
        });
        Ok(stats)
    }

    pub fn apply_sort(stats: &mut [FileStats], config: &AppConfig) {
        if config.total_only || config.summary_only { return; }
        // 安定ソートを「最後のキーから」適用
        for (key, desc) in config.sort_specs.iter().rev() {
            stats.sort_by(|a, b| {
                let ord = match key {
                    SortKey::Lines => a.lines.cmp(&b.lines),
                    SortKey::Chars => a.chars.cmp(&b.chars),
                    SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
                    SortKey::Name => a.path.cmp(&b.path),
                    SortKey::Ext  => a.ext.cmp(&b.ext),
                };
                if *desc { ord.reverse() } else { ord }
            });
        }
    }

    fn is_text_ok(entry: &Entry, config: &AppConfig) -> bool { if !config.text_only { true } else { entry.meta.is_text } }

    fn measure(entry: &Entry, config: &AppConfig) -> Option<FileStats> {
        if !is_text_ok(entry, config) { return None; }
        if config.count_newlines_in_chars { measure_whole(&entry.path, entry, config) } else { measure_by_lines(&entry.path, entry, config) }
    }

    fn measure_whole(path: &std::path::Path, entry: &Entry, config: &AppConfig) -> Option<FileStats> {
        use std::fs::File; use std::io::Read;
        let mut f = File::open(path).ok()?; let mut buf = Vec::new(); f.read_to_end(&mut buf).ok()?;
        if config.text_only && buf.contains(&0) { return None; }
        let s = String::from_utf8_lossy(&buf); let bytes = s.as_bytes();
        let nl = bytecount::count(bytes, b'\n');
        let lines = if bytes.is_empty() { 0 } else if bytes.last() == Some(&b'\n') { nl } else { nl + 1 };
        let chars = s.chars().count();
        let words = config.words.then(|| s.split_whitespace().count());
        let st = FileStats { path: path.to_path_buf(), lines, chars, words, size: entry.meta.size, mtime: entry.meta.mtime, ext: entry.meta.ext.clone(), name: entry.meta.name.clone() };
        apply_numeric_filters(st, config)
    }

    fn measure_by_lines(path: &std::path::Path, entry: &Entry, config: &AppConfig) -> Option<FileStats> {
        use std::fs::File; use std::io::{BufRead, BufReader};
        let file = File::open(path).ok()?; let mut reader = BufReader::new(file);
        let (mut lines, mut chars, mut words) = (0, 0, 0);
        let mut line = String::new();
        loop { line.clear(); let n = reader.read_line(&mut line).ok()?; if n == 0 { break; }
            if line.ends_with('\n') { line.pop(); if line.ends_with('\r') { line.pop(); } }
            lines += 1; chars += line.chars().count(); if config.words { words += line.split_whitespace().count(); }
        }
        let st = FileStats { path: path.to_path_buf(), lines, chars, words: config.words.then_some(words), size: entry.meta.size, mtime: entry.meta.mtime, ext: entry.meta.ext.clone(), name: entry.meta.name.clone() };
        apply_numeric_filters(st, config)
    }

    fn apply_numeric_filters(stats: FileStats, config: &AppConfig) -> Option<FileStats> {
        if let Some(min) = config.filters.min_lines { if stats.lines < min { return None; } }
        if let Some(max) = config.filters.max_lines { if stats.lines > max { return None; } }
        if let Some(min) = config.filters.min_chars { if stats.chars < min { return None; } }
        if let Some(max) = config.filters.max_chars { if stats.chars > max { return None; } }
        if let Some(min) = config.filters.min_words { if stats.words.unwrap_or(0) < min { return None; } }
        if let Some(max) = config.filters.max_words { if stats.words.unwrap_or(0) > max { return None; } }
        // 式フィルタ
        if let Some(ast) = &config.filters.filter_ast {
            let mut ctx = evalexpr::HashMapContext::new();
            ctx.set_value("lines".into(), Value::Int(stats.lines as i64)).ok()?;
            ctx.set_value("chars".into(), Value::Int(stats.chars as i64)).ok()?;
            ctx.set_value("words".into(), Value::Int(stats.words.unwrap_or(0) as i64)).ok()?;
            ctx.set_value("size".into(), Value::Int(stats.size as i64)).ok()?;
            ctx.set_value("ext".into(), Value::String(stats.ext.clone())).ok()?;
            ctx.set_value("name".into(), Value::String(stats.name.clone())).ok()?;
            if let Some(mt) = stats.mtime { ctx.set_value("mtime".into(), Value::Int(mt.timestamp())).ok()?; }
            let v = ast.eval_boolean_with_context(&ctx).ok()?; if !v { return None; }
        }
        Some(stats)
    }

    // 集計
    #[derive(Debug, Clone)]
    pub struct Group { pub key: String, pub lines: usize, pub chars: usize, pub count: usize }

    pub fn aggregate(stats: &[FileStats], by: &[ByKey]) -> Vec<(String, Vec<Group>)> {
        use std::collections::HashMap;
        if by.is_empty() { return vec![]; }
        let mut results = Vec::new();
        for key in by {
            let mut map: HashMap<String, (usize, usize, usize)> = HashMap::new();
            match key {
                ByKey::Ext => {
                    for s in stats { let k = if s.ext.is_empty() { "(noext)".to_string() } else { s.ext.clone() }; let e = map.entry(k).or_insert((0,0,0)); e.0 += s.lines; e.1 += s.chars; e.2 += 1; }
                    let mut v: Vec<Group> = map.into_iter().map(|(key,(l,c,n))| Group{ key, lines:l, chars:c, count:n }).collect();
                    v.sort_by(|a,b| b.lines.cmp(&a.lines));
                    results.push(("By Extension".to_string(), v));
                }
                ByKey::Dir(depth) => {
                    for s in stats { let k = crate::util::get_dir_key(&s.path, *depth); let e = map.entry(k).or_insert((0,0,0)); e.0 += s.lines; e.1 += s.chars; e.2 += 1; }
                    let mut v: Vec<Group> = map.into_iter().map(|(key,(l,c,n))| Group{ key, lines:l, chars:c, count:n }).collect();
                    v.sort_by(|a,b| b.lines.cmp(&a.lines));
                    results.push((format!("By Directory (depth={depth})"), v));
                }
                ByKey::Mtime(g) => {
                    for s in stats { if let Some(mt) = s.mtime { let k = crate::util::mtime_bucket(mt, *g); let e = map.entry(k).or_insert((0,0,0)); e.0 += s.lines; e.1 += s.chars; e.2 += 1; } }
                    let mut v: Vec<Group> = map.into_iter().map(|(key,(l,c,n))| Group{ key, lines:l, chars:c, count:n }).collect();
                    v.sort_by(|a,b| b.lines.cmp(&a.lines));
                    results.push(("By Mtime".to_string(), v));
                }
            }
        }
        results
    }
}

// -------------------------------------------------------------------------------------
// Output
// -------------------------------------------------------------------------------------
mod output {
    use std::io::Write;

    use crate::config::AppConfig;
    use crate::types as t;

    pub fn emit(stats: &[t::Stats], config: &AppConfig) -> anyhow::Result<()> {
        let mut writer: Box<dyn Write> = if let Some(path) = &config.output { Box::new(std::io::BufWriter::new(std::fs::File::create(path)?)) } else { Box::new(std::io::BufWriter::new(std::io::stdout())) };
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

    fn totals(stats: &[t::Stats]) -> (usize, usize, usize) {
        stats.iter().fold((0,0,0), |(l,c,w), s| (l + s.lines, c + s.chars, w + s.words.unwrap_or(0)))
    }

    fn limited<'a>(stats: &'a [t::Stats], config: &AppConfig) -> &'a [t::Stats] {
        let limit = config.top_n.unwrap_or(stats.len()).min(stats.len()); &stats[..limit]
    }

    fn ratio(val: usize, total: usize) -> String { if total==0 { "0.0".into() } else { format!("{:.1}", (val as f64) * 100.0 / (total as f64)) } }

    fn output_table(stats: &[t::Stats], config: &AppConfig, out: &mut impl Write) -> anyhow::Result<()> {
        if config.total_only { return output_summary(stats, config, out); }
        writeln!(out)?; if config.words {
            if config.ratio { writeln!(out, "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\t   WORDS\tFILE")?; }
            else { writeln!(out, "    LINES\t CHARACTERS\t   WORDS\tFILE")?; }
        } else {
            if config.ratio { writeln!(out, "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\tFILE")?; }
            else { writeln!(out, "    LINES\t CHARACTERS\tFILE")?; }
        }
        writeln!(out, "----------------------------------------------")?;
        let (tl, tc, _) = totals(stats);
        for s in limited(stats, config) {
            let path = crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref());
            if config.words {
                if config.ratio { writeln!(out, "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{:>7}\t{}", ratio(s.lines, tl), s.lines, ratio(s.chars, tc), s.chars, s.words.unwrap_or(0), path)?; }
                else { writeln!(out, "{:>10}\t{:>10}\t{:>7}\t{}", s.lines, s.chars, s.words.unwrap_or(0), path)?; }
            } else {
                if config.ratio { writeln!(out, "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{}", ratio(s.lines, tl), s.lines, ratio(s.chars, tc), s.chars, path)?; }
                else { writeln!(out, "{:>10}\t{:>10}\t{}", s.lines, s.chars, path)?; }
            }
        }
        writeln!(out, "---")?;

        if !config.total_only {
            let groups = crate::compute::aggregate(stats, &config.by_modes);
            for (label, mut rows) in groups {
                writeln!(out, "[{label}]")?;
                writeln!(out, "{:>10}\t{:>10}\tKEY", "LINES", "CHARACTERS")?;
                if let Some(n) = config.by_limit { rows.truncate(n); }
                for g in rows { writeln!(out, "{:>10}\t{:>10}\t{} ({} files)", g.lines, g.chars, g.key, g.count)?; }
                writeln!(out, "---")?;
            }
        }
        output_summary(stats, config, out)
    }

    fn output_summary(stats: &[t::Stats], config: &AppConfig, out: &mut impl Write) -> anyhow::Result<()> {
        let (total_lines, total_chars, total_words) = totals(stats);
        if config.words { writeln!(out, "{:>10}\t{:>10}\t{:>7}\tTOTAL ({} files)\n", total_lines, total_chars, total_words, stats.len())?; }
        else { writeln!(out, "{:>10}\t{:>10}\tTOTAL ({} files)\n", total_lines, total_chars, stats.len())?; }
        Ok(())
    }

    fn output_delimited(stats: &[t::Stats], config: &AppConfig, sep: char, out: &mut impl Write) -> anyhow::Result<()> {
        let header = if config.words { format!("lines{sep}chars{sep}words{sep}file") } else { format!("lines{sep}chars{sep}file") };
        writeln!(out, "{header}")?;
        for s in limited(stats, config) {
            let path = crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref());
            if config.words { writeln!(out, "{}{}{}{}{}{}{}", s.lines, sep, s.chars, sep, s.words.unwrap_or(0), sep, escape_if_needed(&path, sep))?; }
            else { writeln!(out, "{}{}{}{}{}", s.lines, sep, s.chars, sep, escape_if_needed(&path, sep))?; }
        }
        if config.total_row {
            let (tl, tc, tw) = totals(stats);
            if config.words { writeln!(out, "{}{}{}{}{}{}{}", tl, sep, tc, sep, tw, sep, escape_if_needed("TOTAL", sep))?; }
            else { writeln!(out, "{}{}{}{}{}", tl, sep, tc, sep, escape_if_needed("TOTAL", sep))?; }
        }
        Ok(())
    }

    fn output_markdown(stats: &[t::Stats], config: &AppConfig, out: &mut impl Write) -> anyhow::Result<()> {
        if config.words {
            if config.ratio { writeln!(out, "| LINES% | LINES | CHARS% | CHARS | WORDS | FILE |\n|---:|---:|---:|---:|---:|:---|")?; }
            else { writeln!(out, "| LINES | CHARS | WORDS | FILE |\n|---:|---:|---:|:---|")?; }
        } else {
            if config.ratio { writeln!(out, "| LINES% | LINES | CHARS% | CHARS | FILE |\n|---:|---:|---:|---:|:---|")?; }
            else { writeln!(out, "| LINES | CHARS | FILE |\n|---:|---:|:---|")?; }
        }
        let (tl, tc, _) = totals(stats);
        for s in limited(stats, config) {
            let path = crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref()).replace('|', "\\|");
            if config.words {
                if config.ratio { writeln!(out, "| {} | {} | {} | {} | {} | {} |", ratio(s.lines, tl), s.lines, ratio(s.chars, tc), s.chars, s.words.unwrap_or(0), path)?; }
                else { writeln!(out, "| {} | {} | {} | {} |", s.lines, s.chars, s.words.unwrap_or(0), path)?; }
            } else {
                if config.ratio { writeln!(out, "| {} | {} | {} | {} | {} |", ratio(s.lines, tl), s.lines, ratio(s.chars, tc), s.chars, path)?; }
                else { writeln!(out, "| {} | {} | {} |", s.lines, s.chars, path)?; }
            }
        }
        // 集計
        let groups = crate::compute::aggregate(stats, &config.by_modes);
        for (label, mut rows) in groups {
            writeln!(out, "\n### {label}\n")?;
            writeln!(out, "| LINES | CHARS | KEY | COUNT |\n|---:|---:|:---|---:|")?;
            if let Some(n) = config.by_limit { rows.truncate(n); }
            for g in rows { writeln!(out, "| {} | {} | {} | {} |", g.lines, g.chars, g.key.replace('|', "\\|"), g.count)?; }
        }
        Ok(())
    }

    fn output_json(stats: &[t::Stats], config: &AppConfig, out: &mut impl Write) -> anyhow::Result<()> {
        let files: Vec<t::OutFile> = stats.iter().map(|s| t::OutFile {
            file: crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref()),
            lines: s.lines, chars: s.chars, words: s.words, size: s.size,
            mtime: s.mtime.map(|d| d.to_rfc3339()), ext: s.ext.clone(),
        }).collect();
        let (tl, tc, tw) = totals(stats);
        let summary = t::OutSummary { lines: tl, chars: tc, words: config.words.then_some(tw), files: stats.len() };
        let mut groups_json = Vec::new();
        for (label, rows) in crate::compute::aggregate(stats, &config.by_modes) {
            let mut jr = Vec::new();
            let mut rows = rows; if let Some(n)=config.by_limit { rows.truncate(n); }
            for g in rows { jr.push(t::OutGroupRow { key: g.key, lines: g.lines, chars: g.chars, count: g.count }); }
            groups_json.push(t::OutGroup { label, rows: jr });
        }
        let out_json = t::Out { files, summary, by: if groups_json.is_empty(){None}else{Some(groups_json)} };
        serde_json::to_writer_pretty(&mut *out, &out_json)?; writeln!(out)?; Ok(())
    }

    fn output_yaml(stats: &[t::Stats], config: &AppConfig, out: &mut impl Write) -> anyhow::Result<()> {
        use serde_yaml as y;
        let files: Vec<t::OutFile> = stats.iter().map(|s| t::OutFile {
            file: crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref()),
            lines: s.lines, chars: s.chars, words: s.words, size: s.size,
            mtime: s.mtime.map(|d| d.to_rfc3339()), ext: s.ext.clone(),
        }).collect();
        let (tl, tc, tw) = totals(stats);
        let summary = t::OutSummary { lines: tl, chars: tc, words: config.words.then_some(tw), files: stats.len() };
        let mut groups_yaml = Vec::new();
        for (label, rows) in crate::compute::aggregate(stats, &config.by_modes) {
            let mut jr = Vec::new();
            let mut rows = rows; if let Some(n)=config.by_limit { rows.truncate(n); }
            for g in rows { jr.push(t::OutGroupRow { key: g.key, lines: g.lines, chars: g.chars, count: g.count }); }
            groups_yaml.push(t::OutGroup { label, rows: jr });
        }
        let out_yaml = t::Out { files, summary, by: if groups_yaml.is_empty(){None}else{Some(groups_yaml)} };
        let s = y::to_string(&out_yaml)?; writeln!(out, "{}", s)?; Ok(())
    }

    fn output_jsonl(stats: &[t::Stats], config: &AppConfig, out: &mut impl Write) -> anyhow::Result<()> {
        for s in stats {
            let item = serde_json::json!({
                "type": "file",
                "file": crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref()),
                "lines": s.lines, "chars": s.chars, "words": s.words, "size": s.size,
                "mtime": s.mtime.map(|d| d.to_rfc3339()), "ext": s.ext,
            });
            serde_json::to_writer(&mut *out, &item)?; writeln!(out)?;
        }
        let (tl, tc, tw) = totals(stats);
        let total = serde_json::json!({ "type": "total", "lines": tl, "chars": tc, "words": if config.words { Some(tw) } else { None }, "files": stats.len() });
        serde_json::to_writer(&mut *out, &total)?; writeln!(out)?; Ok(())
    }

    #[inline]
    fn escape_if_needed(s: &str, sep: char) -> String { if sep == ',' { let escaped = s.replace('"', "\"\""); format!("\"{escaped}\"") } else { s.to_string() } }
}

// -------------------------------------------------------------------------------------
// Compare (simple)
// -------------------------------------------------------------------------------------
mod compare {
    use anyhow::Result;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct Summary { lines: usize, chars: usize, words: Option<usize>, files: usize }
    #[derive(Debug, Deserialize)]
    struct FileItem { file: String, lines: usize, chars: usize, words: Option<usize> }
    #[derive(Debug, Deserialize)]
    struct Snapshot { files: Vec<FileItem>, summary: Summary }

    pub fn run(old_path: &std::path::Path, new_path: &std::path::Path) -> Result<String> {
        let old: Snapshot = serde_json::from_reader(std::fs::File::open(old_path)?)?;
        let new: Snapshot = serde_json::from_reader(std::fs::File::open(new_path)?)?;
        use std::collections::HashMap;
        let mut om: HashMap<&str, &FileItem> = HashMap::new();
        for f in &old.files { om.insert(&f.file, f); }
        let mut out = String::new();
        out.push_str("DIFF (new - old)\n");
        out.push_str(&format!("Lines: {} -> {} (Δ {})\n", old.summary.lines, new.summary.lines, (new.summary.lines as isize - old.summary.lines as isize)));
        out.push_str(&format!("Chars: {} -> {} (Δ {})\n", old.summary.chars, new.summary.chars, (new.summary.chars as isize - old.summary.chars as isize)));
        if let (Some(ow), Some(nw)) = (old.summary.words, new.summary.words) { out.push_str(&format!("Words: {} -> {} (Δ {})\n", ow, nw, (nw as isize - ow as isize))); }
        out.push_str("\n[Changed files]\n");
        for nf in &new.files {
            if let Some(of) = om.get(nf.file.as_str()) {
                let dl = nf.lines as isize - of.lines as isize; let dc = nf.chars as isize - of.chars as isize;
                if dl != 0 || dc != 0 { out.push_str(&format!("{:>10} L  {:>10} C  {}\n", dl, dc, nf.file)); }
            } else {
                out.push_str(&format!("{:>10} L  {:>10} C  {} (added)\n", nf.lines as isize, nf.chars as isize, nf.file));
            }
        }
        Ok(out)
    }
}

// -------------------------------------------------------------------------------------
// main
// -------------------------------------------------------------------------------------
fn main() -> Result<()> {
    let args = cli::Args::parse();
    let config = config::AppConfig::try_from(args)?;

    if let Some((old, newp)) = &config.compare { // 比較モード
        let diff = compare::run(old, newp).context("compare failed")?;
        println!("{}", diff);
        return Ok(());
    }

    if !matches!(config.format, cli::OutputFormat::Json) && atty::is(Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", VERSION, config.jobs);
    }

    // 進捗（簡易）
    if config.progress { eprintln!("[count_lines] scanning & measuring..."); }

    // 測定
    let mut stats = match compute::process_entries(&config) {
        Ok(v) => v,
        Err(e) => {
            if config.strict { return Err(e).context("failed to measure entries"); }
            eprintln!("[warn] {}", e);
            Vec::new()
        }
    };

    // ソート & トップN
    compute::apply_sort(&mut stats, &config);

    // 出力
    output::emit(&stats, &config).context("failed to emit output")?;

    Ok(())
}