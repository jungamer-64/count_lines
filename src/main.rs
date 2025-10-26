// src/main.rs
#![allow(clippy::multiple_crate_versions)]

use anyhow::Result;
use atty::Stream;
use clap::Parser;
use std::path::PathBuf;

use chrono::{DateTime, Local};

const VERSION: &str = "2.2.0";

// ==========================
// CLI (Args / Enums)
// ==========================
mod cli {
    use super::*;

    #[derive(Debug, Clone, clap::ValueEnum)]
    pub enum OutputFormat {
        Table,
        Csv,
        Tsv,
        Json,
    }

    #[derive(Debug, Clone, clap::ValueEnum)]
    pub enum SortKey {
        Lines,
        Chars,
        Words,
        Name,
        Ext,
    }

    #[derive(Parser, Debug)]
    #[command(name = "count_lines", version = VERSION, about = "ファイル行数/文字数/単語数の集計ツール")]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Args {
        /// 出力フォーマット
        #[arg(long, value_enum, default_value = "table")]
        pub format: OutputFormat,

        /// ソートキー
        #[arg(long, value_enum, default_value = "lines")]
        pub sort: SortKey,

        /// 降順ソート
        #[arg(long)]
        pub desc: bool,

        /// 上位N件のみ表示
        #[arg(long)]
        pub top: Option<usize>,

        /// サマリ軸 (ext, dir, dir=N)
        #[arg(long)]
        pub by: Option<String>,

        /// サマリのみ表示（一覧は出力しないが By 集計は出す）
        #[arg(long)]
        pub summary_only: bool,

        /// 合計のみ表示（一覧と By 集計は出さない）
        #[arg(long)]
        pub total_only: bool,

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

        /// 最大ファイルサイズ
        #[arg(long)]
        pub max_size: Option<String>,

        /// 最小ファイルサイズ
        #[arg(long)]
        pub min_size: Option<String>,

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

        /// 色なし（現状は装飾未使用）
        #[arg(long)]
        pub no_color: bool,

        /// CSV/TSV 末尾に TOTAL 行を出力
        #[arg(long)]
        pub total_row: bool,

        /// 指定日時以降
        #[arg(long)]
        pub mtime_since: Option<String>,

        /// 指定日時以前
        #[arg(long)]
        pub mtime_until: Option<String>,

        /// 改行も文字数に含める（直感的カウント）
        #[arg(long)]
        pub count_newlines_in_chars: bool,

        /// 対象パス
        pub paths: Vec<PathBuf>,
    }
}

// ==========================
// Config / Modes / Filters
// ==========================
mod config {
    use super::*;

    use crate::cli::{OutputFormat, SortKey};

    #[derive(Debug)]
    pub enum ByMode {
        None,
        Ext,
        Dir(usize),
    }

    #[derive(Debug, Default)]
    #[allow(clippy::struct_field_names)]
    pub struct Filters {
        pub include_patterns: Vec<glob::Pattern>,
        pub exclude_patterns: Vec<glob::Pattern>,
        pub include_paths: Vec<glob::Pattern>,
        pub exclude_paths: Vec<glob::Pattern>,
        pub exclude_dirs: Vec<glob::Pattern>,
        pub ext_filters: Vec<String>,
        pub max_size: Option<u64>,
        pub min_size: Option<u64>,
        pub min_lines: Option<usize>,
        pub max_lines: Option<usize>,
        pub min_chars: Option<usize>,
        pub max_chars: Option<usize>,
        pub min_words: Option<usize>,
        pub max_words: Option<usize>,
    }

    #[derive(Debug)]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Config {
        pub format: OutputFormat,
        pub sort_key: SortKey,
        pub sort_desc: bool,
        pub top_n: Option<usize>,
        pub by_mode: ByMode,
        pub summary_only: bool,
        pub total_only: bool,
        pub filters: Filters,
        pub hidden: bool,
        pub follow: bool,
        pub use_git: bool,
        pub jobs: usize,
        pub no_default_prune: bool,
        pub abs_path: bool,
        pub abs_canonical: bool,
        pub trim_root: Option<PathBuf>,
        #[allow(dead_code)]
        pub no_color: bool,
        pub words: bool,
        pub count_newlines_in_chars: bool,
        pub text_only: bool,
        pub files_from: Option<PathBuf>,
        pub files_from0: Option<PathBuf>,
        pub paths: Vec<PathBuf>,
        pub mtime_since: Option<DateTime<Local>>,
        pub mtime_until: Option<DateTime<Local>>,
        pub total_row: bool,
    }

    impl Config {
        pub fn from_args(args: crate::cli::Args) -> anyhow::Result<Self> {
            let by_mode = match args.by {
                None => ByMode::None,
                Some(ref s) if s == "ext" => ByMode::Ext,
                Some(ref s) if s.starts_with("dir") => {
                    let depth = s.strip_prefix("dir=")
                        .and_then(|d| d.parse().ok())
                        .unwrap_or(1);
                    ByMode::Dir(depth)
                }
                Some(ref s) => anyhow::bail!("Unknown --by mode: {s}"),
            };

            let filters = Filters {
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
                max_size: args.max_size.as_ref().and_then(|s| crate::util::parse_size(s).ok()),
                min_size: args.min_size.as_ref().and_then(|s| crate::util::parse_size(s).ok()),
                min_lines: args.min_lines,
                max_lines: args.max_lines,
                min_chars: args.min_chars,
                max_chars: args.max_chars,
                min_words: args.min_words,
                max_words: args.max_words,
            };

            let jobs = args.jobs.unwrap_or_else(num_cpus::get);
            let paths = if args.paths.is_empty() {
                vec![PathBuf::from(".")]
            } else {
                args.paths
            };

            let mtime_since = args
                .mtime_since
                .as_ref()
                .and_then(|s| crate::util::parse_datetime(s).ok());
            let mtime_until = args
                .mtime_until
                .as_ref()
                .and_then(|s| crate::util::parse_datetime(s).ok());

            Ok(Self {
                format: args.format,
                sort_key: args.sort,
                sort_desc: args.desc,
                top_n: args.top,
                by_mode,
                summary_only: args.summary_only,
                total_only: args.total_only,
                filters,
                hidden: args.hidden,
                follow: args.follow,
                use_git: args.git,
                jobs,
                no_default_prune: args.no_default_prune,
                abs_path: args.abs_path,
                abs_canonical: args.abs_canonical,
                trim_root: args.trim_root,
                no_color: args.no_color,
                words: args.words,
                count_newlines_in_chars: args.count_newlines_in_chars,
                text_only: args.text_only,
                files_from: args.files_from,
                files_from0: args.files_from0,
                paths,
                mtime_since,
                mtime_until,
                total_row: args.total_row,
            })
        }
    }

    pub use Config as AppConfig;
    pub use Filters as AppFilters;
}

// ==========================
// Domain types (stats / JSON)
// ==========================
mod types {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Clone)]
    pub struct FileStats {
        pub path: PathBuf,
        pub lines: usize,
        pub chars: usize,
        pub words: Option<usize>,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonOutput {
        pub files: Vec<JsonFile>,
        pub summary: JsonSummary,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub by_extension: Option<Vec<JsonByExt>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub by_directory: Option<Vec<JsonByDir>>,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonFile {
        pub file: String,
        pub lines: usize,
        pub chars: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub words: Option<usize>,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonSummary {
        pub lines: usize,
        pub chars: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub words: Option<usize>,
        pub files: usize,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonByExt {
        pub ext: String,
        pub lines: usize,
        pub chars: usize,
        pub count: usize,
    }

    #[derive(Debug, Serialize)]
    pub struct JsonByDir {
        pub dir: String,
        pub lines: usize,
        pub chars: usize,
        pub count: usize,
    }

    pub use FileStats as Stats;
    pub use JsonByDir as OutByDir;
    pub use JsonByExt as OutByExt;
    pub use JsonFile as OutFile;
    pub use JsonOutput as Out;
    pub use JsonSummary as OutSummary;
}

// ==========================
// Utilities
// ==========================
mod util {
    use super::*;
    use anyhow::Context as _;
    use std::path::{Path, PathBuf};

    pub fn parse_patterns(patterns: &[String]) -> anyhow::Result<Vec<glob::Pattern>> {
        patterns
            .iter()
            .map(|p| glob::Pattern::new(p).with_context(|| format!("Invalid pattern: {p}")))
            .collect()
    }

    pub fn parse_size(s: &str) -> anyhow::Result<u64> {
        let s = s.trim().replace('_', "");

        let parse_with_suffix = |suffixes: &[&str], multiplier: u64| {
            for suffix in suffixes {
                if let Some(stripped) = s.strip_suffix(suffix) {
                    return Some((stripped, multiplier));
                }
            }
            None
        };

        let (num_str, multiplier) = parse_with_suffix(&["KiB", "KB", "K", "k"], 1024)
            .or_else(|| parse_with_suffix(&["MiB", "MB", "M", "m"], 1024 * 1024))
            .or_else(|| parse_with_suffix(&["GiB", "GB", "G", "g"], 1024 * 1024 * 1024))
            .or_else(|| parse_with_suffix(&["TiB", "TB", "T", "t"], 1024 * 1024 * 1024 * 1024))
            .unwrap_or((s.as_str(), 1));

        let num: u64 = num_str.parse().context("Invalid size number")?;
        Ok(num * multiplier)
    }

    pub fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Local>> {
        use chrono::{NaiveDate, NaiveDateTime, TimeZone};

        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Local));
        }
        if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
            return Local
                .from_local_datetime(&ndt)
                .single()
                .context("Ambiguous datetime");
        }
        if let Ok(nd) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            let ndt = nd.and_hms_opt(0, 0, 0).context("Invalid time")?;
            return Local
                .from_local_datetime(&ndt)
                .single()
                .context("Ambiguous datetime");
        }
        anyhow::bail!("Cannot parse datetime: {s}")
    }

    pub fn logical_absolute(path: &Path) -> PathBuf {
        if path.is_absolute() {
            return path.to_path_buf();
        }
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|_| path.to_path_buf())
    }

    pub fn format_path(path: &Path, abs_path: bool, abs_canonical: bool, trim_root: &Option<PathBuf>) -> String {
        let mut path = if abs_path {
            if abs_canonical {
                path.canonicalize().unwrap_or_else(|_| logical_absolute(path))
            } else {
                logical_absolute(path)
            }
        } else {
            path.to_path_buf()
        };

        if let Some(root) = trim_root {
            if let Ok(stripped) = path.strip_prefix(root) {
                path = stripped.to_path_buf();
            }
        }

        path.display().to_string()
    }

    pub fn get_dir_key(path: &Path, depth: usize) -> String {
        // ファイルは parent() に寄せてから components を数える
        let mut p = path;
        if path.file_name().is_some() {
            if let Some(parent) = path.parent() {
                p = parent;
            }
        }
        // 以降は Normal ディレクトリのみカウント
        use std::path::Component;
        let mut parts = Vec::new();
        for comp in p.components() {
            if let Component::Normal(s) = comp {
                parts.push(s.to_string_lossy().into_owned());
                if parts.len() >= depth {
                    break;
                }
            }
        }
        if parts.is_empty() {
            ".".to_string()
        } else {
            parts.join("/")
        }
    }
}

// ==========================
// File selection (git / walk)
// ==========================
mod files {
    use super::*;
    use walkdir::WalkDir;

    use crate::config::{AppConfig, AppFilters};

    const DEFAULT_PRUNE_DIRS: &[&str] = &[
        ".git", ".hg", ".svn", "node_modules", ".venv", "venv", "build", "dist", "target",
        ".cache", ".direnv", ".mypy_cache", ".pytest_cache", "coverage", "__pycache__", ".idea",
        ".next", ".nuxt",
    ];

    pub fn collect_files(config: &AppConfig) -> anyhow::Result<Vec<PathBuf>> {
        if let Some(ref from0) = config.files_from0 {
            return read_files_from0(from0);
        }
        if let Some(ref from) = config.files_from {
            return read_files_from(from);
        }

        if config.use_git {
            if let Ok(files) = collect_git_files(config) {
                return Ok(files);
            }
        }

        collect_find_files(config)
    }

    fn read_files_from(path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(reader
            .lines()
            .map_while(Result::ok)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .collect())
    }

    fn read_files_from0(path: &std::path::Path) -> anyhow::Result<Vec<PathBuf>> {
        use std::fs::File;
        use std::io::Read;
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let mut out = Vec::new();
        for chunk in buf.split(|b| *b == 0) {
            if chunk.is_empty() {
                continue;
            }
            let s = String::from_utf8_lossy(chunk);
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                out.push(PathBuf::from(trimmed));
            }
        }
        Ok(out)
    }

    fn collect_git_files(config: &AppConfig) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for root in &config.paths {
            let output = std::process::Command::new("git")
                .arg("ls-files")
                .arg("-z")
                .arg("--cached")
                .arg("--others")
                .arg("--exclude-standard")
                .arg("--")
                .arg(root)
                .current_dir(root)
                .output()?;

            if !output.status.success() {
                anyhow::bail!("git ls-files failed");
            }

            for chunk in output.stdout.split(|b| *b == 0) {
                if chunk.is_empty() {
                    continue;
                }
                let s = String::from_utf8_lossy(chunk);
                let s = s.trim();
                if s.is_empty() {
                    continue;
                }
                files.push(root.join(s));
            }
        }
        Ok(files)
    }

    fn should_process_entry(entry: &walkdir::DirEntry, config: &AppConfig) -> bool {
        let path = entry.path();

        // 隠しファイル/ディレクトリの除外
        if !config.hidden {
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    return false;
                }
            }
        }

        // デフォルト剪定
        if !config.no_default_prune && entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            if DEFAULT_PRUNE_DIRS.contains(&name.as_ref()) {
                return false;
            }
        }

        // 除外ディレクトリ
        if entry.file_type().is_dir() {
            for pattern in &config.filters.exclude_dirs {
                if pattern.matches_path(path) {
                    return false;
                }
            }
        }

        true
    }

    fn matches_filters(path: &std::path::Path, config: &AppConfig) -> bool {
        let AppFilters {
            include_patterns,
            exclude_patterns,
            include_paths,
            exclude_paths,
            ext_filters,
            max_size,
            min_size,
            ..
        } = &config.filters;

        // ファイル名 (include)
        if !include_patterns.is_empty() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !include_patterns.iter().any(|p| p.matches(&name)) {
                return false;
            }
        }

        // ファイル名 (exclude)
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if exclude_patterns.iter().any(|p| p.matches(&name)) {
            return false;
        }

        // パス (include)
        if !include_paths.is_empty() && !include_paths.iter().any(|p| p.matches_path(path)) {
            return false;
        }

        // パス (exclude)
        if exclude_paths.iter().any(|p| p.matches_path(path)) {
            return false;
        }

        // 拡張子
        if !ext_filters.is_empty() {
            if let Some(ext) = path.extension() {
                let ext_lower = ext.to_string_lossy().to_lowercase();
                if !ext_filters.contains(&ext_lower) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // サイズ/mtime（metadata は 1 度だけ取得）
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Some(max_size) = max_size {
                if metadata.len() > *max_size {
                    return false;
                }
            }
            if let Some(min_size) = min_size {
                if metadata.len() < *min_size {
                    return false;
                }
            }
            if let Ok(modified) = metadata.modified() {
                let modified: DateTime<Local> = modified.into();
                if let Some(since) = &config.mtime_since {
                    if modified < *since {
                        return false;
                    }
                }
                if let Some(until) = &config.mtime_until {
                    if modified > *until {
                        return false;
                    }
                }
            }
        }

        true
    }

    pub fn collect_find_files(config: &AppConfig) -> anyhow::Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for root in &config.paths {
            let walker = WalkDir::new(root)
                .follow_links(config.follow)
                .into_iter()
                .filter_entry(|e| should_process_entry(e, config));

            for entry in walker {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                if !matches_filters(path, config) {
                    continue;
                }
                files.push(path.to_path_buf());
            }
        }

        Ok(files)
    }
}

// ==========================
// Measuring / Sorting
// ==========================
mod compute {
    use super::*;
    use rayon::prelude::*;

    use crate::cli::SortKey;
    use crate::config::AppConfig;
    use crate::types::Stats as FileStats;

    const READ_BUF_BYTES: usize = 8192;

    pub fn process_files(config: &AppConfig) -> anyhow::Result<Vec<FileStats>> {
        let files = crate::files::collect_files(config)?;

        // Use a dedicated pool here instead of setting a global one.
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.jobs)
            .build()?;

        let stats: Vec<FileStats> = pool.install(|| {
            files
                .par_iter()
                .filter_map(|path| measure_file(path, config))
                .collect()
        });

        Ok(stats)
    }

    pub fn sort_stats(stats: &mut [FileStats], config: &AppConfig) {
        stats.sort_by(|a, b| {
            let cmp = match config.sort_key {
                SortKey::Lines => a.lines.cmp(&b.lines),
                SortKey::Chars => a.chars.cmp(&b.chars),
                SortKey::Words => a.words.unwrap_or(0).cmp(&b.words.unwrap_or(0)),
                SortKey::Name => a.path.cmp(&b.path),
                SortKey::Ext => {
                    let ext_a = a.path.extension().unwrap_or_default();
                    let ext_b = b.path.extension().unwrap_or_default();
                    ext_a.cmp(ext_b)
                }
            };
            if config.sort_desc { cmp.reverse() } else { cmp }
        });
    }

    fn is_text_file(path: &std::path::Path) -> bool {
        use std::fs::File;
        use std::io::Read;
        let Ok(mut file) = File::open(path) else { return false; };
        let mut buffer = [0u8; READ_BUF_BYTES];
        let n = file.read(&mut buffer).unwrap_or(0);
        // NULL バイトがあればバイナリと判定
        !buffer[..n].contains(&0)
    }

    fn measure_file(path: &std::path::Path, config: &AppConfig) -> Option<FileStats> {
        use std::fs::File;
        use std::io::{BufRead, BufReader, Read};

        // 改行も文字数に含めるモード：一括読みで正確にカウント
        if config.count_newlines_in_chars {
            let mut f = File::open(path).ok()?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).ok()?;

            if config.text_only && buf.contains(&0) {
                return None;
            }

            let s = String::from_utf8_lossy(&buf);
            let bytes = s.as_bytes();

            // 行数：'\n' の数 +（末尾が改行でなければ +1、空なら 0）
            let nl = bytes.iter().filter(|&&b| b == b'\n').count();
            let lines = if bytes.is_empty() {
                0
            } else if bytes.last() == Some(&b'\n') {
                nl
            } else {
                nl + 1
            };

            // 文字数：改行含む
            let chars = s.chars().count();

            let words = if config.words { s.split_whitespace().count() } else { 0 };

            let stats = FileStats { path: path.to_path_buf(), lines, chars, words: if config.words { Some(words) } else { None } };
            return apply_numeric_filters(stats, config);
        }

        // 従来モード：行ごと読み（改行は文字数に含めない）
        if config.text_only && !is_text_file(path) {
            return None;
        }
        let file = File::open(path).ok()?;
        let reader = BufReader::new(file);

        let mut lines = 0;
        let mut chars = 0;
        let mut words = 0;

        for line in reader.lines() {
            let line = line.ok()?;
            lines += 1;
            chars += line.chars().count();
            if config.words {
                words += line.split_whitespace().count();
            }
        }

        let stats = FileStats { path: path.to_path_buf(), lines, chars, words: if config.words { Some(words) } else { None } };
        apply_numeric_filters(stats, config)
    }

    fn apply_numeric_filters(stats: FileStats, config: &AppConfig) -> Option<FileStats> {
        if let Some(min) = config.filters.min_lines {
            if stats.lines < min {
                return None;
            }
        }
        if let Some(max) = config.filters.max_lines {
            if stats.lines > max {
                return None;
            }
        }
        if let Some(min) = config.filters.min_chars {
            if stats.chars < min {
                return None;
            }
        }
        if let Some(max) = config.filters.max_chars {
            if stats.chars > max {
                return None;
            }
        }
        if let Some(min) = config.filters.min_words {
            if stats.words.unwrap_or(0) < min {
                return None;
            }
        }
        if let Some(max) = config.filters.max_words {
            if stats.words.unwrap_or(0) > max {
                return None;
            }
        }
        Some(stats)
    }
}

// ==========================
// Output formatting
// ==========================
mod output {
    use std::collections::HashMap;

    use crate::cli::OutputFormat;
    use crate::config::{AppConfig, ByMode};
    use crate::types as t;

    pub fn emit(stats: &[t::Stats], config: &AppConfig) -> anyhow::Result<()> {
        match config.format {
            OutputFormat::Json => output_json(stats, config)?,
            OutputFormat::Csv => output_delimited(stats, config, ',')?,
            OutputFormat::Tsv => output_delimited(stats, config, '\t')?,
            OutputFormat::Table => output_table(stats, config)?,
        }
        Ok(())
    }

    fn output_table(stats: &[t::Stats], config: &AppConfig) -> anyhow::Result<()> {
        // total-only: 一覧も by も出さず、最後のサマリだけ
        if config.total_only {
            output_summary(stats, config);
            return Ok(());
        }

        // summary-only でないときだけ一覧を出す
        if !config.summary_only {
            println!();
            if config.words {
                println!("    LINES\t CHARACTERS\t   WORDS\tFILE");
            } else {
                println!("    LINES\t CHARACTERS\tFILE");
            }
            println!("----------------------------------------------");

            for stat in limited(stats, config) {
                let path = crate::util::format_path(&stat.path, config.abs_path, config.abs_canonical, &config.trim_root);
                if config.words {
                    println!("{:10}\t{:10}\t{:7}\t{}", stat.lines, stat.chars, stat.words.unwrap_or(0), path);
                } else {
                    println!("{:10}\t{:10}\t{}", stat.lines, stat.chars, path);
                }
            }
            println!("---");
        }

        // “summary-only” でも By 集計は出す（一覧のみ省略）
        if !config.total_only {
            match config.by_mode {
                ByMode::Ext => output_group(&aggregate_by_extension(stats), "[By Extension]", "EXT", None)?,
                ByMode::Dir(depth) => output_group(&aggregate_by_directory(stats, depth), "[By Directory]", &format!("DIR (depth={depth})"), None)?,
                ByMode::None => {}
            }
        }

        output_summary(stats, config);
        Ok(())
    }

    fn limited<'a>(stats: &'a [t::Stats], config: &AppConfig) -> &'a [t::Stats] {
        let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
        &stats[..limit]
    }

    fn output_group(groups: &[Group], title: &str, key_label: &str, limit: Option<usize>) -> anyhow::Result<()> {
        println!("{title}");
        println!("{:>10}\t{:>10}\t{key_label}", "LINES", "CHARACTERS");

        let iter = groups.iter().take(limit.unwrap_or(groups.len()));
        for g in iter {
            println!("{:10}\t{:10}\t{} ({} files)", g.lines, g.chars, g.key, g.count);
        }
        println!("---");
        Ok(())
    }

    fn output_summary(stats: &[t::Stats], config: &AppConfig) {
        let (total_lines, total_chars, total_words) = totals(stats);

        if config.words {
            println!(
                "{:10}\t{:10}\t{:7}\tTOTAL ({} files)\n",
                total_lines,
                total_chars,
                total_words,
                stats.len()
            );
        } else {
            println!(
                "{:10}\t{:10}\tTOTAL ({} files)\n",
                total_lines,
                total_chars,
                stats.len()
            );
        }
    }

    fn output_delimited(stats: &[t::Stats], config: &AppConfig, sep: char) -> anyhow::Result<()> {
        let header = if config.words {
            format!("lines{sep}chars{sep}words{sep}file")
        } else {
            format!("lines{sep}chars{sep}file")
        };
        println!("{}", header);

        for stat in limited(stats, config) {
            let path = crate::util::format_path(&stat.path, config.abs_path, config.abs_canonical, &config.trim_root);
            if config.words {
                println!(
                    "{}{}{}{}{}{}{}",
                    stat.lines,
                    sep,
                    stat.chars,
                    sep,
                    stat.words.unwrap_or(0),
                    sep,
                    escape_if_needed(&path, sep)
                );
            } else {
                println!(
                    "{}{}{}{}{}",
                    stat.lines,
                    sep,
                    stat.chars,
                    sep,
                    escape_if_needed(&path, sep)
                );
            }
        }

        if config.total_row {
            let (total_lines, total_chars, total_words) = totals(stats);
            if config.words {
                println!(
                    "{}{}{}{}{}{}{}",
                    total_lines,
                    sep,
                    total_chars,
                    sep,
                    total_words,
                    sep,
                    escape_if_needed("TOTAL", sep)
                );
            } else {
                println!(
                    "{}{}{}{}{}",
                    total_lines,
                    sep,
                    total_chars,
                    sep,
                    escape_if_needed("TOTAL", sep)
                );
            }
        }
        Ok(())
    }

    fn escape_if_needed(s: &str, sep: char) -> String {
        if sep == ',' {
            let escaped = s.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        } else {
            s.to_string()
        }
    }

    fn output_json(stats: &[t::Stats], config: &AppConfig) -> anyhow::Result<()> {
        let files: Vec<t::OutFile> = stats
            .iter()
            .map(|s| t::OutFile {
                file: crate::util::format_path(&s.path, config.abs_path, config.abs_canonical, &config.trim_root),
                lines: s.lines,
                chars: s.chars,
                words: s.words,
            })
            .collect();

        let (total_lines, total_chars, total_words) = totals(stats);

        let summary = t::OutSummary {
            lines: total_lines,
            chars: total_chars,
            words: if config.words { Some(total_words) } else { None },
            files: stats.len(),
        };

        let by_extension = match config.by_mode {
            ByMode::Ext => Some(
                aggregate_by_extension(stats)
                    .into_iter()
                    .map(|g| t::OutByExt { ext: if g.key.is_empty() { "(noext)".to_string() } else { g.key }, lines: g.lines, chars: g.chars, count: g.count })
                    .collect(),
            ),
            _ => None,
        };

        let by_directory = match config.by_mode {
            ByMode::Dir(depth) => Some(
                aggregate_by_directory(stats, depth)
                    .into_iter()
                    .map(|g| t::OutByDir { dir: g.key, lines: g.lines, chars: g.chars, count: g.count })
                    .collect(),
            ),
            _ => None,
        };

        let output = t::Out { files, summary, by_extension, by_directory };
        println!("{}", serde_json::to_string_pretty(&output)?);
        Ok(())
    }

    // ----- helpers -----

    #[derive(Debug, Clone)]
    struct Group {
        key: String,
        lines: usize,
        chars: usize,
        count: usize,
    }

    fn totals(stats: &[t::Stats]) -> (usize, usize, usize) {
        let total_lines: usize = stats.iter().map(|s| s.lines).sum();
        let total_chars: usize = stats.iter().map(|s| s.chars).sum();
        let total_words: usize = stats.iter().filter_map(|s| s.words).sum();
        (total_lines, total_chars, total_words)
    }

    fn aggregate_by_extension(stats: &[t::Stats]) -> Vec<Group> {
        let mut by_ext: HashMap<String, (usize, usize, usize)> = HashMap::new();
        for stat in stats {
            let ext = stat
                .path
                .extension()
                .map_or_else(String::new, |e| e.to_string_lossy().to_lowercase());
            let entry = by_ext.entry(ext).or_insert((0, 0, 0));
            entry.0 += stat.lines;
            entry.1 += stat.chars;
            entry.2 += 1;
        }
        let mut v: Vec<Group> = by_ext
            .into_iter()
            .map(|(key, (lines, chars, count))| Group { key, lines, chars, count })
            .collect();
        v.sort_by(|a, b| b.lines.cmp(&a.lines));
        v
    }

    fn aggregate_by_directory(stats: &[t::Stats], depth: usize) -> Vec<Group> {
        let mut by_dir: HashMap<String, (usize, usize, usize)> = HashMap::new();
        for stat in stats {
            let dir_key = crate::util::get_dir_key(&stat.path, depth);
            let entry = by_dir.entry(dir_key).or_insert((0, 0, 0));
            entry.0 += stat.lines;
            entry.1 += stat.chars;
            entry.2 += 1;
        }
        let mut v: Vec<Group> = by_dir
            .into_iter()
            .map(|(key, (lines, chars, count))| Group { key, lines, chars, count })
            .collect();
        v.sort_by(|a, b| b.lines.cmp(&a.lines));
        v
    }
}

// ==========================
// Main
// ==========================
fn main() -> Result<()> {
    let args = cli::Args::parse();
    let config = config::AppConfig::from_args(args)?;

    if !matches!(config.format, cli::OutputFormat::Json) && atty::is(Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", VERSION, config.jobs);
    }

    let mut stats = compute::process_files(&config)?;
    compute::sort_stats(&mut stats, &config);

    output::emit(&stats, &config)?;

    Ok(())
}
