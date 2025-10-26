// src/main.rs
#![allow(clippy::multiple_crate_versions)]

use anyhow::{Context, Result};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use clap::Parser;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const VERSION: &str = "2.2.0";

#[derive(Parser, Debug)]
#[command(name = "count_lines", version = VERSION, about = "ファイル行数/文字数/単語数の集計ツール")]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// 出力フォーマット
    #[arg(long, value_enum, default_value = "table")]
    format: OutputFormat,

    /// ソートキー
    #[arg(long, value_enum, default_value = "lines")]
    sort: SortKey,

    /// 降順ソート
    #[arg(long)]
    desc: bool,

    /// 上位N件のみ表示
    #[arg(long)]
    top: Option<usize>,

    /// サマリ軸 (ext, dir, dir=N)
    #[arg(long)]
    by: Option<String>,

    /// サマリのみ表示
    #[arg(long)]
    summary_only: bool,

    /// 合計のみ表示
    #[arg(long)]
    total_only: bool,

    /// 含めるファイル名パターン
    #[arg(long)]
    include: Vec<String>,

    /// 除外するファイル名パターン
    #[arg(long)]
    exclude: Vec<String>,

    /// 含めるパスパターン
    #[arg(long)]
    include_path: Vec<String>,

    /// 除外するパスパターン
    #[arg(long)]
    exclude_path: Vec<String>,

    /// 除外ディレクトリパターン
    #[arg(long)]
    exclude_dir: Vec<String>,

    /// 拡張子フィルタ (カンマ区切り)
    #[arg(long)]
    ext: Option<String>,

    /// 最大ファイルサイズ
    #[arg(long)]
    max_size: Option<String>,

    /// 最小行数
    #[arg(long)]
    min_lines: Option<usize>,

    /// 最大行数
    #[arg(long)]
    max_lines: Option<usize>,

    /// 最小文字数
    #[arg(long)]
    min_chars: Option<usize>,

    /// 最大文字数
    #[arg(long)]
    max_chars: Option<usize>,

    /// 単語数も計測
    #[arg(long)]
    words: bool,

    /// 最小単語数
    #[arg(long)]
    min_words: Option<usize>,

    /// 最大単語数
    #[arg(long)]
    max_words: Option<usize>,

    /// テキストファイルのみ
    #[arg(long)]
    text_only: bool,

    /// ファイル一覧を読み込む (改行区切り)
    #[arg(long)]
    files_from: Option<PathBuf>,

    /// 隠しファイルも対象
    #[arg(long)]
    hidden: bool,

    /// シンボリックリンクを辿る
    #[arg(long)]
    follow: bool,

    /// .gitignore を尊重
    #[arg(long)]
    git: bool,

    /// 並列数
    #[arg(long)]
    jobs: Option<usize>,

    /// 既定の剪定を無効化
    #[arg(long)]
    no_default_prune: bool,

    /// 絶対パス出力
    #[arg(long)]
    abs_path: bool,

    /// パス先頭を削除
    #[arg(long)]
    trim_root: Option<PathBuf>,

    /// 色なし
    #[arg(long)]
    no_color: bool,

    /// 指定日時以降
    #[arg(long)]
    mtime_since: Option<String>,

    /// 指定日時以前
    #[arg(long)]
    mtime_until: Option<String>,

    /// 対象パス
    paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Table,
    Csv,
    Tsv,
    Json,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum SortKey {
    Lines,
    Chars,
    Words,
    Name,
    Ext,
}

#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Config {
    format: OutputFormat,
    sort_key: SortKey,
    sort_desc: bool,
    top_n: Option<usize>,
    by_mode: ByMode,
    summary_only: bool,
    total_only: bool,
    filters: Filters,
    hidden: bool,
    follow: bool,
    use_git: bool,
    jobs: usize,
    no_default_prune: bool,
    abs_path: bool,
    trim_root: Option<PathBuf>,
    #[allow(dead_code)]
    no_color: bool,
    words: bool,
    text_only: bool,
    files_from: Option<PathBuf>,
    paths: Vec<PathBuf>,
    mtime_since: Option<DateTime<Local>>,
    mtime_until: Option<DateTime<Local>>,
}

#[derive(Debug)]
enum ByMode {
    None,
    Ext,
    Dir(usize),
}

#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)]
struct Filters {
    include_patterns: Vec<glob::Pattern>,
    exclude_patterns: Vec<glob::Pattern>,
    include_paths: Vec<glob::Pattern>,
    exclude_paths: Vec<glob::Pattern>,
    exclude_dirs: Vec<glob::Pattern>,
    ext_filters: Vec<String>,
    max_size: Option<u64>,
    min_lines: Option<usize>,
    max_lines: Option<usize>,
    min_chars: Option<usize>,
    max_chars: Option<usize>,
    min_words: Option<usize>,
    max_words: Option<usize>,
}

#[derive(Debug, Clone)]
struct FileStats {
    path: PathBuf,
    lines: usize,
    chars: usize,
    words: Option<usize>,
}

#[derive(Debug, Serialize)]
struct JsonOutput {
    files: Vec<JsonFile>,
    summary: JsonSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_extension: Option<Vec<JsonByExt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    by_directory: Option<Vec<JsonByDir>>,
}

#[derive(Debug, Serialize)]
struct JsonFile {
    file: String,
    lines: usize,
    chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    words: Option<usize>,
}

#[derive(Debug, Serialize)]
struct JsonSummary {
    lines: usize,
    chars: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    words: Option<usize>,
    files: usize,
}

#[derive(Debug, Serialize)]
struct JsonByExt {
    ext: String,
    lines: usize,
    chars: usize,
    count: usize,
}

#[derive(Debug, Serialize)]
struct JsonByDir {
    dir: String,
    lines: usize,
    chars: usize,
    count: usize,
}

impl Config {
    fn from_args(args: Args) -> Result<Self> {
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
            include_patterns: parse_patterns(&args.include)?,
            exclude_patterns: parse_patterns(&args.exclude)?,
            include_paths: parse_patterns(&args.include_path)?,
            exclude_paths: parse_patterns(&args.exclude_path)?,
            exclude_dirs: parse_patterns(&args.exclude_dir)?,
            ext_filters: args.ext
                .as_ref()
                .map(|s| s.split(',').map(|e| e.trim().to_lowercase()).collect())
                .unwrap_or_default(),
            max_size: args.max_size.as_ref().and_then(|s| parse_size(s).ok()),
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

        let mtime_since = args.mtime_since.as_ref().and_then(|s| parse_datetime(s).ok());
        let mtime_until = args.mtime_until.as_ref().and_then(|s| parse_datetime(s).ok());

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
            trim_root: args.trim_root,
            no_color: args.no_color,
            words: args.words,
            text_only: args.text_only,
            files_from: args.files_from,
            paths,
            mtime_since,
            mtime_until,
        })
    }
}

fn parse_patterns(patterns: &[String]) -> Result<Vec<glob::Pattern>> {
    patterns
        .iter()
        .map(|p| glob::Pattern::new(p).context("Invalid pattern"))
        .collect()
}

fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();
    
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
        .unwrap_or((s, 1));

    let num: u64 = num_str.parse().context("Invalid size number")?;
    Ok(num * multiplier)
}

fn parse_datetime(s: &str) -> Result<DateTime<Local>> {
    // 簡易的な日時パース (ISO8601形式など)
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Local));
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Local.from_local_datetime(&ndt).single().context("Ambiguous datetime");
    }
    if let Ok(nd) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let ndt = nd.and_hms_opt(0, 0, 0).context("Invalid time")?;
        return Local.from_local_datetime(&ndt).single().context("Ambiguous datetime");
    }
    anyhow::bail!("Cannot parse datetime: {s}")
}

fn collect_files(config: &Config) -> Result<Vec<PathBuf>> {
    if let Some(ref from_file) = config.files_from {
        return read_files_from(from_file);
    }

    if config.use_git {
        if let Ok(files) = collect_git_files(config) {
            return Ok(files);
        }
    }

    collect_find_files(config)
}

fn read_files_from(path: &Path) -> Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader
        .lines()
        .map_while(Result::ok)
        .filter(|line| !line.trim().is_empty())
        .map(PathBuf::from)
        .collect())
}

fn collect_git_files(config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for root in &config.paths {
        let output = std::process::Command::new("git")
            .arg("ls-files")
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

        let stdout = String::from_utf8_lossy(&output.stdout);
        files.extend(
            stdout
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| root.join(line)),
        );
    }
    Ok(files)
}

fn collect_find_files(config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for root in &config.paths {
        let walker = WalkDir::new(root)
            .follow_links(config.follow)
            .into_iter()
            .filter_entry(|e| should_process_entry(e, config));

        for entry in walker {
            let entry = entry?;
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

fn should_process_entry(entry: &walkdir::DirEntry, config: &Config) -> bool {
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
        let default_prune = [
            ".git", ".hg", ".svn", "node_modules", ".venv", "venv",
            "build", "dist", "target", ".cache", ".direnv",
            ".mypy_cache", ".pytest_cache", "coverage", "__pycache__",
            ".idea", ".next", ".nuxt",
        ];
        if default_prune.contains(&name.as_ref()) {
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

fn matches_filters(path: &Path, config: &Config) -> bool {
    let filters = &config.filters;

    // ファイル名パターン (include)
    if !filters.include_patterns.is_empty() {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if !filters.include_patterns.iter().any(|p| p.matches(&name)) {
            return false;
        }
    }

    // ファイル名パターン (exclude)
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if filters.exclude_patterns.iter().any(|p| p.matches(&name)) {
        return false;
    }

    // パスパターン (include)
    if !filters.include_paths.is_empty() {
        if !filters.include_paths.iter().any(|p| p.matches_path(path)) {
            return false;
        }
    }

    // パスパターン (exclude)
    if filters.exclude_paths.iter().any(|p| p.matches_path(path)) {
        return false;
    }

    // 拡張子フィルタ
    if !filters.ext_filters.is_empty() {
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if !filters.ext_filters.contains(&ext_lower) {
                return false;
            }
        } else {
            return false;
        }
    }

    // サイズフィルタ
    if let Some(max_size) = filters.max_size {
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > max_size {
                return false;
            }
        }
    }

    // mtime フィルタ
    #[allow(clippy::collapsible_if)]
    if let Ok(metadata) = std::fs::metadata(path) {
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

fn measure_file(path: &Path, config: &Config) -> Option<FileStats> {
    // テキストファイルチェック
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

    let stats = FileStats {
        path: path.to_path_buf(),
        lines,
        chars,
        words: if config.words { Some(words) } else { None },
    };

    // 行数・文字数・単語数フィルタ
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

fn is_text_file(path: &Path) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };

    let mut buffer = [0u8; 8192];
    let n = file.read(&mut buffer).unwrap_or(0);

    // NULL バイトがあればバイナリと判定
    !buffer[..n].contains(&0)
}

fn process_files(config: &Config) -> Result<Vec<FileStats>> {
    let files = collect_files(config)?;

    rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build_global()
        .ok();

    let stats: Vec<FileStats> = files
        .par_iter()
        .filter_map(|path| measure_file(path, config))
        .collect();

    Ok(stats)
}

fn format_path(path: &Path, config: &Config) -> String {
    let mut path = if config.abs_path {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    if let Some(ref trim_root) = config.trim_root {
        if let Ok(stripped) = path.strip_prefix(trim_root) {
            path = stripped.to_path_buf();
        }
    }

    path.display().to_string()
}

fn sort_stats(stats: &mut [FileStats], config: &Config) {
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
        if config.sort_desc {
            cmp.reverse()
        } else {
            cmp
        }
    });
}

fn output_table(stats: &[FileStats], config: &Config) {
    println!();
    if config.words {
        println!("    LINES\t CHARACTERS\t   WORDS\tFILE");
    } else {
        println!("    LINES\t CHARACTERS\tFILE");
    }
    println!("----------------------------------------------");

    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    for stat in &stats[..limit] {
        let path = format_path(&stat.path, config);
        if config.words {
            println!(
                "{:10}\t{:10}\t{:7}\t{}",
                stat.lines,
                stat.chars,
                stat.words.unwrap_or(0),
                path
            );
        } else {
            println!("{:10}\t{:10}\t{}", stat.lines, stat.chars, path);
        }
    }
    println!("---");
}

fn output_by_extension(stats: &[FileStats], _config: &Config) {
    let mut by_ext: HashMap<String, (usize, usize, usize)> = HashMap::new();

    for stat in stats {
        let ext = stat
            .path
            .extension()
            .map_or_else(|| "(noext)".to_string(), |e| e.to_string_lossy().to_lowercase());

        let entry = by_ext.entry(ext).or_insert((0, 0, 0));
        entry.0 += stat.lines;
        entry.1 += stat.chars;
        entry.2 += 1;
    }

    println!("[By Extension]");
    println!("{:>10}\t{:>10}\tEXT", "LINES", "CHARACTERS");

    let mut entries: Vec<_> = by_ext.into_iter().collect();
    entries.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    for (ext, (lines, chars, count)) in entries {
        println!("{lines:10}\t{chars:10}\t{ext} ({count} files)");
    }
    println!("---");
}

fn output_by_directory(stats: &[FileStats], _config: &Config, depth: usize) {
    let mut by_dir: HashMap<String, (usize, usize, usize)> = HashMap::new();

    for stat in stats {
        let dir_key = get_dir_key(&stat.path, depth);
        let entry = by_dir.entry(dir_key).or_insert((0, 0, 0));
        entry.0 += stat.lines;
        entry.1 += stat.chars;
        entry.2 += 1;
    }

    println!("[By Directory]");
    println!(
        "{:>10}\t{:>10}\tDIR (depth={depth})",
        "LINES", "CHARACTERS"
    );

    let mut entries: Vec<_> = by_dir.into_iter().collect();
    entries.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    for (dir, (lines, chars, count)) in entries {
        println!("{lines:10}\t{chars:10}\t{dir} ({count} files)");
    }
    println!("---");
}

fn get_dir_key(path: &Path, depth: usize) -> String {
    let components: Vec<_> = path.components().take(depth).collect();
    components
        .iter()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn output_summary(stats: &[FileStats], config: &Config) {
    let total_lines: usize = stats.iter().map(|s| s.lines).sum();
    let total_chars: usize = stats.iter().map(|s| s.chars).sum();
    let total_words: usize = stats.iter().filter_map(|s| s.words).sum();

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

fn output_json(stats: &[FileStats], config: &Config) -> Result<()> {
    let files: Vec<JsonFile> = stats
        .iter()
        .map(|s| JsonFile {
            file: format_path(&s.path, config),
            lines: s.lines,
            chars: s.chars,
            words: s.words,
        })
        .collect();

    let total_lines: usize = stats.iter().map(|s| s.lines).sum();
    let total_chars: usize = stats.iter().map(|s| s.chars).sum();
    let total_words: usize = stats.iter().filter_map(|s| s.words).sum();

    let summary = JsonSummary {
        lines: total_lines,
        chars: total_chars,
        words: if config.words { Some(total_words) } else { None },
        files: stats.len(),
    };

    let by_extension = match config.by_mode {
        ByMode::Ext => {
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
            Some(
                by_ext
                    .into_iter()
                    .map(|(ext, (lines, chars, count))| JsonByExt {
                        ext: if ext.is_empty() { "(noext)".to_string() } else { ext },
                        lines,
                        chars,
                        count,
                    })
                    .collect(),
            )
        }
        _ => None,
    };

    let by_directory = match config.by_mode {
        ByMode::Dir(depth) => {
            let mut by_dir: HashMap<String, (usize, usize, usize)> = HashMap::new();
            for stat in stats {
                let dir_key = get_dir_key(&stat.path, depth);
                let entry = by_dir.entry(dir_key).or_insert((0, 0, 0));
                entry.0 += stat.lines;
                entry.1 += stat.chars;
                entry.2 += 1;
            }
            Some(
                by_dir
                    .into_iter()
                    .map(|(dir, (lines, chars, count))| JsonByDir {
                        dir,
                        lines,
                        chars,
                        count,
                    })
                    .collect(),
            )
        }
        _ => None,
    };

    let output = JsonOutput {
        files,
        summary,
        by_extension,
        by_directory,
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn output_csv(stats: &[FileStats], config: &Config) {
    if config.words {
        println!("lines,chars,words,file");
    } else {
        println!("lines,chars,file");
    }

    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    for stat in &stats[..limit] {
        let path = format_path(&stat.path, config);
        let escaped_path = path.replace('"', "\"\"");
        if config.words {
            println!(
                "{},{},{},\"{}\"",
                stat.lines,
                stat.chars,
                stat.words.unwrap_or(0),
                escaped_path
            );
        } else {
            println!("{},{},\"{}\"", stat.lines, stat.chars, escaped_path);
        }
    }
}

fn output_tsv(stats: &[FileStats], config: &Config) {
    if config.words {
        println!("lines\tchars\twords\tfile");
    } else {
        println!("lines\tchars\tfile");
    }

    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    for stat in &stats[..limit] {
        let path = format_path(&stat.path, config);
        if config.words {
            println!(
                "{}\t{}\t{}\t{}",
                stat.lines,
                stat.chars,
                stat.words.unwrap_or(0),
                path
            );
        } else {
            println!("{}\t{}\t{}", stat.lines, stat.chars, path);
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_args(args)?;

    if !matches!(config.format, OutputFormat::Json) && atty::is(atty::Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", VERSION, config.jobs);
    }

    let mut stats = process_files(&config)?;
    sort_stats(&mut stats, &config);

    match config.format {
        OutputFormat::Json => {
            output_json(&stats, &config)?;
        }
        OutputFormat::Csv => {
            output_csv(&stats, &config);
        }
        OutputFormat::Tsv => {
            output_tsv(&stats, &config);
        }
        OutputFormat::Table => {
            if !config.total_only && !config.summary_only {
                output_table(&stats, &config);
            }

            if !config.total_only {
                match config.by_mode {
                    ByMode::Ext => output_by_extension(&stats, &config),
                    ByMode::Dir(depth) => output_by_directory(&stats, &config, depth),
                    ByMode::None => {}
                }
            }

            output_summary(&stats, &config);
        }
    }

    Ok(())
}