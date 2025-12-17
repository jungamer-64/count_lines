// crates/infra/src/measurement/measurer.rs
//! ファイル計測のリファクタリング版

#[cfg(feature = "parallel")]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[cfg(feature = "eval")]
use evalexpr::{ContextWithMutableVariables, HashMapContext, Value};
#[cfg(feature = "parallel")]
use rayon::prelude::*;

#[cfg(feature = "eval")]
use count_lines_domain::config::FilterAst;
use count_lines_domain::{
    config::Config,
    model::{FileEntry, FileStats, FileStatsV2, MeasurementOutcome},
};
#[cfg(feature = "eval")]
use count_lines_shared_kernel::DomainError;
use count_lines_shared_kernel::{InfrastructureError, Result};

use crate::{cache::CacheStore, measurement::strategies::measure_by_lines};

// Helper type aliases to simplify signatures
type PendingEntry = (usize, String, FileEntry);
///
/// # Errors
///
/// 失敗した計測や入出力処理があった場合に `Err` を返します。
pub fn measure_entries(entries: Vec<FileEntry>, config: &Config) -> Result<MeasurementOutcome> {
    if entries.is_empty() {
        return Ok(MeasurementOutcome::new(Vec::new(), Vec::new(), Vec::new()));
    }

    if config.incremental {
        return measure_entries_incremental(entries, config);
    }

    let stats = measure_all(entries, config)?;
    let changed = stats.iter().map(|s| s.path.clone()).collect();
    Ok(MeasurementOutcome::new(stats, changed, Vec::new()))
}

fn measure_entries_incremental(
    entries: Vec<FileEntry>,
    config: &Config,
) -> Result<MeasurementOutcome> {
    let mut cache = CacheStore::load(config)?;

    let (mut processed, pending) = FileMeasurer::collect_cached_entries(entries, &cache, config)?;

    let changed_files = if pending.is_empty() {
        Vec::new()
    } else {
        let (mut new_processed, new_changed) =
            FileMeasurer::measure_pending_entries(pending, config)?;
        processed.append(&mut new_processed);
        new_changed
    };

    processed.sort_by_key(|r| r.index);

    let mut retain = HashSet::new();
    let mut results = Vec::with_capacity(processed.len());
    for record in processed {
        retain.insert(record.key.clone());
        cache.update(
            record.key.clone(),
            &record.entry,
            &record.stats,
            config.cache_verify,
        );
        results.push(record.stats);
    }

    let removed_keys = cache.prune_except(&retain);
    let removed_files = removed_keys.into_iter().map(PathBuf::from).collect();
    if let Err(err) = cache.save() {
        eprintln!("[warn] failed to persist cache: {err}");
    }

    Ok(MeasurementOutcome::new(
        results,
        changed_files,
        removed_files,
    ))
}

#[cfg(feature = "parallel")]
fn measure_all(entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
    if entries.len() < 10 || config.jobs == 1 {
        return measure_sequential(entries, config);
    }
    measure_parallel(entries, config)
}

#[cfg(not(feature = "parallel"))]
fn measure_all(entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
    let _ = config;
    measure_sequential(entries, config)
}

struct IndexedResult {
    index: usize,
    key: String,
    entry: FileEntry,
    stats: FileStats,
}

/// 順次処理版
fn measure_sequential(entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
    let mut stats = Vec::with_capacity(entries.len());
    let mut failure_count = 0usize;

    for entry in entries {
        match FileMeasurer::measure(&entry, config) {
            Ok(Some(stat)) => stats.push(stat),
            Ok(None) => {} // フィルタリングされた
            Err(e) => {
                if config.strict {
                    return Err(e);
                }
                failure_count += 1;
            }
        }
    }

    if config.progress && failure_count > 0 {
        eprintln!("[warn] {failure_count} files failed measurement");
    }

    Ok(stats)
}

/// 並列処理版
#[cfg(feature = "parallel")]
fn measure_parallel(entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
    let progress = config
        .progress
        .then(|| (AtomicUsize::new(0), entries.len()));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build()
        .map_err(|e| InfrastructureError::ThreadPoolCreation {
            details: e.to_string(),
        })?;

    let results: Vec<_> = pool.install(|| {
        entries
            .into_par_iter()
            .map(|entry| process_entry(entry, config, progress.as_ref()))
            .collect()
    });

    collect_parallel_results(results, config)
}

#[cfg(feature = "parallel")]
fn process_entry(
    entry: FileEntry,
    config: &Config,
    progress: Option<&(AtomicUsize, usize)>,
) -> (std::path::PathBuf, Result<Option<FileStats>>) {
    let result = FileMeasurer::measure(&entry, config);

    if let Some((counter, total)) = progress {
        let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
        if current % 100 == 0 || current == *total {
            eprintln!("[{current}/{total}] Processing...");
        }
    }

    (entry.path, result)
}

/// Aggregate results returned by parallel workers.
///
/// Returns a Vec<FileStats> containing successful measurements. If `config.strict` is true
/// this will return early with the first error encountered. When `config.progress` is true
/// warnings are emitted for failed measurements.
#[cfg(feature = "parallel")]
fn collect_parallel_results(
    results: Vec<(std::path::PathBuf, Result<Option<FileStats>>)>,
    config: &Config,
) -> Result<Vec<FileStats>> {
    let mut stats = Vec::new();
    let mut failure_count = 0usize;

    for (path, result) in results {
        match result {
            Ok(Some(stat)) => stats.push(stat),
            Ok(None) => {}
            Err(e) => {
                if config.strict {
                    return Err(e);
                }
                if config.progress {
                    eprintln!(
                        "[warn] measurement failed for {path}: {err}",
                        path = path.display(),
                        err = e
                    );
                }
                failure_count += 1;
            }
        }
    }

    if config.progress && failure_count > 0 {
        eprintln!("[warn] {failure_count} files failed measurement");
    }

    Ok(stats)
}

/// ファイル計測ロジック
struct FileMeasurer;

impl FileMeasurer {
    fn measure(entry: &FileEntry, config: &Config) -> Result<Option<FileStats>> {
        // テキストファイルフィルタ
        if config.text_only && !entry.meta.is_text {
            return Ok(None);
        }

        // 計測実行
        let stats = Self::compute_stats(entry, config)?;

        // フィルタ適用
        let filtered = Self::apply_filters(stats, config)?;
        Ok(filtered.map(FileStats::from_v2))
    }

    fn compute_stats(entry: &FileEntry, config: &Config) -> Result<FileStatsV2> {
        measure_by_lines(&entry.path, &entry.meta, config).ok_or_else(|| {
            InfrastructureError::MeasurementError {
                path: entry.path.clone(),
                reason: "failed to measure file".to_string(),
            }
            .into()
        })
    }

    fn apply_filters(stats: FileStatsV2, config: &Config) -> Result<Option<FileStatsV2>> {
        let filters = &config.filters;

        // 行数フィルタ
        if !filters.lines_range.contains(stats.lines().value()) {
            return Ok(None);
        }

        // 文字数フィルタ
        if !filters.chars_range.contains(stats.chars().value()) {
            return Ok(None);
        }

        // 単語数フィルタ
        if let Some(words) = stats.words()
            && !filters.words_range.contains(words.value())
        {
            return Ok(None);
        }

        // 式フィルタ
        #[cfg(feature = "eval")]
        {
            if let Some(ast) = &filters.filter_ast
                && !Self::eval_filter(&stats, ast)?
            {
                return Ok(None);
            }
        }

        Ok(Some(stats))
    }

    #[cfg(feature = "eval")]
    fn eval_filter(stats: &FileStatsV2, ast: &FilterAst) -> Result<bool> {
        let ctx = Self::build_eval_context(stats)?;
        ast.eval_boolean_with_context(&ctx)
            .map_err(|e: evalexpr::EvalexprError| {
                DomainError::InvalidFilterExpression {
                    expression: String::new(),
                    details: e.to_string(),
                }
                .into()
            })
    }
    /// Set an integer variable in the eval context from a `usize` value.
    /// Performs safe conversion to `i64` and returns a `DomainError` on overflow or context errors.
    #[cfg(feature = "eval")]
    fn set_int_from_usize(ctx: &mut HashMapContext, key: &str, val: usize) -> Result<()> {
        use std::convert::TryFrom;
        let v = i64::try_from(val).map_err(|_| DomainError::InvalidFilterExpression {
            expression: String::new(),
            details: format!("numeric overflow for {key}"),
        })?;
        ctx.set_value(key.into(), Value::Int(v)).map_err(|e| {
            DomainError::InvalidFilterExpression {
                expression: String::new(),
                details: e.to_string(),
            }
        })?;
        Ok(())
    }

    /// Set an integer variable in the eval context from a `u64` value.
    /// Performs safe conversion to `i64` and returns a `DomainError` on overflow or context errors.
    #[cfg(feature = "eval")]
    fn set_int_from_u64(ctx: &mut HashMapContext, key: &str, val: u64) -> Result<()> {
        use std::convert::TryFrom;
        let v = i64::try_from(val).map_err(|_| DomainError::InvalidFilterExpression {
            expression: String::new(),
            details: format!("numeric overflow for {key}"),
        })?;
        ctx.set_value(key.into(), Value::Int(v)).map_err(|e| {
            DomainError::InvalidFilterExpression {
                expression: String::new(),
                details: e.to_string(),
            }
        })?;
        Ok(())
    }

    /// Set an integer variable in the eval context from an i64 value.
    #[cfg(feature = "eval")]
    fn set_int_direct(ctx: &mut HashMapContext, key: &str, val: i64) -> Result<()> {
        ctx.set_value(key.into(), Value::Int(val)).map_err(|e| {
            DomainError::InvalidFilterExpression {
                expression: String::new(),
                details: e.to_string(),
            }
        })?;
        Ok(())
    }

    /// Set a string variable in the eval context.
    #[cfg(feature = "eval")]
    fn set_string(ctx: &mut HashMapContext, key: &str, val: &str) -> Result<()> {
        ctx.set_value(key.into(), Value::String(val.to_string()))
            .map_err(|e| DomainError::InvalidFilterExpression {
                expression: String::new(),
                details: e.to_string(),
            })?;
        Ok(())
    }

    /// Populate the eval context with variables derived from `FileStatsV2`.
    /// This delegates to numeric and string population helpers.
    #[cfg(feature = "eval")]
    fn populate_eval_context(ctx: &mut HashMapContext, stats: &FileStatsV2) -> Result<()> {
        Self::populate_numeric_vars(ctx, stats)?;
        Self::populate_string_vars(ctx, stats)?;
        Ok(())
    }

    /// Populate numeric variables (lines, chars, words, size, mtime) into the eval context.
    #[cfg(feature = "eval")]
    fn populate_numeric_vars(ctx: &mut HashMapContext, stats: &FileStatsV2) -> Result<()> {
        Self::set_int_from_usize(ctx, "lines", stats.lines().value())?;
        Self::set_int_from_usize(ctx, "chars", stats.chars().value())?;

        let words_val = stats
            .words()
            .map_or(0usize, count_lines_domain::value_objects::WordCount::value);
        Self::set_int_from_usize(ctx, "words", words_val)?;

        Self::set_int_from_u64(ctx, "size", stats.size().bytes())?;

        if let Some(mtime) = stats.mtime() {
            Self::set_int_direct(ctx, "mtime", mtime.timestamp().timestamp())?;
        }

        Ok(())
    }

    /// Populate string variables (ext, name) into the eval context.
    #[cfg(feature = "eval")]
    fn populate_string_vars(ctx: &mut HashMapContext, stats: &FileStatsV2) -> Result<()> {
        Self::set_string(ctx, "ext", stats.ext().as_str())?;
        Self::set_string(ctx, "name", stats.name().as_str())?;
        Ok(())
    }

    #[cfg(feature = "eval")]
    fn build_eval_context(stats: &FileStatsV2) -> Result<HashMapContext> {
        let mut ctx = HashMapContext::new();
        Self::populate_eval_context(&mut ctx, stats)?;
        Ok(ctx)
    }

    fn collect_cached_entries(
        entries: Vec<FileEntry>,
        cache: &CacheStore,
        config: &Config,
    ) -> Result<(Vec<IndexedResult>, Vec<PendingEntry>)> {
        let mut processed: Vec<IndexedResult> = Vec::with_capacity(entries.len());
        let mut pending: Vec<(usize, String, FileEntry)> = Vec::new();

        for (index, entry) in entries.into_iter().enumerate() {
            if config.text_only && !entry.meta.is_text {
                continue;
            }

            let key = CacheStore::path_key(&entry.path);
            if let Some(mut stats) =
                cache.get_if_fresh(&key, &entry, config.words, config.cache_verify)
            {
                if let Some(filtered) = Self::apply_filters(stats.to_v2(), config)? {
                    stats = FileStats::from_v2(filtered);
                    processed.push(IndexedResult {
                        index,
                        key,
                        entry,
                        stats,
                    });
                } else {
                    // filtered out
                }
            } else {
                pending.push((index, key, entry));
            }
        }

        Ok((processed, pending))
    }

    fn measure_pending_entries(
        pending: Vec<PendingEntry>,
        config: &Config,
    ) -> Result<(Vec<IndexedResult>, Vec<PathBuf>)> {
        let measure_input: Vec<FileEntry> =
            pending.iter().map(|(_, _, entry)| entry.clone()).collect();
        let measured = measure_all(measure_input, config)?;
        let mut measured_map: HashMap<PathBuf, FileStats> = measured
            .into_iter()
            .map(|stat| (stat.path.clone(), stat))
            .collect();

        let mut processed: Vec<IndexedResult> = Vec::with_capacity(measured_map.len());
        let mut changed_files: Vec<PathBuf> = Vec::new();

        for (index, key, entry) in pending {
            if let Some(stats) = measured_map.remove(&entry.path) {
                // Clone the path before moving `entry` into the processed vector so
                // we can record the changed file without re-querying `processed`.
                let entry_path = entry.path.clone();
                processed.push(IndexedResult {
                    index,
                    key,
                    entry,
                    stats,
                });
                changed_files.push(entry_path);
            }
        }

        Ok((processed, changed_files))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::{fs, io::Write, path::Path, time::Duration};

    use tempfile::NamedTempFile;

    use super::*;
    use crate::measurement::strategies::measure_by_lines;
    use count_lines_domain::model::FileMeta;

    struct TempFile {
        pub path: std::path::PathBuf,
        // keep the NamedTempFile so it's removed on Drop
        _file: NamedTempFile,
    }

    impl TempFile {
        fn new(content: &str) -> Self {
            let mut ntf = NamedTempFile::new().expect("create temp file");
            ntf.write_all(content.as_bytes()).expect("write temp file");
            ntf.flush().expect("flush temp file");
            let path = ntf.path().to_path_buf();
            Self { path, _file: ntf }
        }
    }

    fn make_meta(path: &Path) -> FileMeta {
        let size = fs::metadata(path).unwrap().len();
        FileMeta {
            size,
            mtime: None,
            is_text: true,
            ext: "txt".to_string(),
            name: "test.txt".to_string(),
        }
    }

    fn make_config() -> Config {
        use count_lines_domain::{config::Filters, options::OutputFormat};

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
            paths: vec![],
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
            watch_output: count_lines_domain::options::WatchOutput::Full,
            compare: None,
        }
    }

    #[test]
    fn measure_by_lines_counts_correctly() {
        let file = TempFile::new("line1\nline2\nline3");
        let mut config = make_config();
        config.words = true;

        let stats = measure_by_lines(&file.path, &make_meta(&file.path), &config).unwrap();

        assert_eq!(stats.lines().value(), 3);
        assert_eq!(stats.chars().value(), 15); // "line1" + "line2" + "line3"
        assert_eq!(stats.words().unwrap().value(), 3);
    }

    #[test]
    fn line_measurement_counts_newlines_when_enabled() {
        let file = TempFile::new("a\nb\n");
        let mut config = make_config();
        config.count_newlines_in_chars = true;

        let stats = measure_by_lines(&file.path, &make_meta(&file.path), &config).unwrap();

        assert_eq!(stats.lines().value(), 2);
        assert_eq!(stats.chars().value(), 4); // 'a' + '\n' + 'b' + '\n'
    }

    #[test]
    fn line_measurement_respects_text_only_flag() {
        let file = TempFile::new("text\0binary");
        let mut config = make_config();
        config.text_only = true;
        config.count_newlines_in_chars = true;

        let result = measure_by_lines(&file.path, &make_meta(&file.path), &config);
        assert!(result.is_none());
    }

    #[test]
    fn sequential_measurement() {
        let file1 = TempFile::new("content1");
        let file2 = TempFile::new("content2\ncontent3");

        let entries = vec![
            FileEntry {
                path: file1.path.clone(),
                meta: make_meta(&file1.path),
            },
            FileEntry {
                path: file2.path.clone(),
                meta: make_meta(&file2.path),
            },
        ];

        let config = make_config();
        let stats = measure_sequential(entries, &config).unwrap();

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].lines, 1);
        assert_eq!(stats[1].lines, 2);
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_measurement() {
        let file1 = TempFile::new("line1\nline2");
        let file2 = TempFile::new("line3\nline4\nline5");

        let entries = vec![
            FileEntry {
                path: file1.path.clone(),
                meta: make_meta(&file1.path),
            },
            FileEntry {
                path: file2.path.clone(),
                meta: make_meta(&file2.path),
            },
        ];

        let mut config = make_config();
        config.jobs = 2;

        let stats = measure_parallel(entries, &config).unwrap();

        assert_eq!(stats.len(), 2);
        // Order may vary in parallel execution
        let total_lines: usize = stats.iter().map(|s| s.lines).sum();
        assert_eq!(total_lines, 5);
    }

    #[cfg(feature = "eval")]
    #[test]
    fn build_eval_context_populates_expected_vars() {
        let file = TempFile::new("line1\nline2\nline3");
        let mut config = make_config();
        config.words = true;

        let stats_v2 = measure_by_lines(&file.path, &make_meta(&file.path), &config).unwrap();
        let ctx = FileMeasurer::build_eval_context(&stats_v2).expect("build context");

        let ast = evalexpr::build_operator_tree("lines == 3 && chars == 15 && words == 3")
            .expect("parse");
        assert!(ast.eval_boolean_with_context(&ctx).expect("eval"));
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn collect_parallel_results_handles_errors() {
        let file1 = TempFile::new("line1\nline2");
        let file2 = TempFile::new("line3\nline4\nline5");

        let mut config = make_config();
        config.words = true;

        let stats1_v2 = measure_by_lines(&file1.path, &make_meta(&file1.path), &config).unwrap();
        let stats1 = FileStats::from_v2(stats1_v2);

        let results = vec![
            (file1.path, Ok(Some(stats1))),
            (
                file2.path.clone(),
                Err(InfrastructureError::MeasurementError {
                    path: file2.path,
                    reason: "simulated".to_string(),
                }
                .into()),
            ),
        ];

        let processed = collect_parallel_results(results, &config).expect("collect");
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].lines, 2);
    }

    #[test]
    fn grapheme_counting_handles_combined_chars() {
        // "é" (e + combining acute accent) is 2 chars but 1 grapheme
        // "👨‍👩‍👧‍👦" (family emoji) is 7 chars but 1 grapheme
        let content = "e\u{0301}\n👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let file = TempFile::new(content);
        let mut config = make_config();
        config.words = true;

        let stats = measure_by_lines(&file.path, &make_meta(&file.path), &config).unwrap();

        assert_eq!(stats.lines().value(), 2);
        // Line 1: "é" (1 grapheme)
        // Line 2: "👨‍👩‍👧‍👦" (1 grapheme)
        // Total: 2 graphemes
        assert_eq!(stats.chars().value(), 2);
    }
}
