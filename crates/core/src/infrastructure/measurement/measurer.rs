// crates/core/src/infrastructure/measurement/measurer.rs
//! ファイル計測のリファクタリング版

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use rayon::prelude::*;

use crate::{
    application::commands::MeasurementOutcome,
    domain::{
        config::Config,
        model::{FileEntry, FileStats, FileStatsV2},
    },
    error::*,
    infrastructure::{
        cache::CacheStore,
        measurement::strategies::{measure_by_lines, measure_entire_file},
    },
};

/// ファイル計測の主要エントリポイント（改善版）
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

fn measure_entries_incremental(entries: Vec<FileEntry>, config: &Config) -> Result<MeasurementOutcome> {
    let mut cache = CacheStore::load(config)?;
    let mut processed: Vec<IndexedResult> = Vec::with_capacity(entries.len());
    let mut pending: Vec<(usize, String, FileEntry)> = Vec::new();
    let mut changed_files = Vec::new();

    for (index, entry) in entries.into_iter().enumerate() {
        if config.text_only && !entry.meta.is_text {
            // テキストのみの場合、非テキストは常に除外
            continue;
        }

        let key = CacheStore::path_key(&entry.path);
        if let Some(mut stats) = cache.get_if_fresh(&key, &entry, config.words, config.cache_verify) {
            match FileMeasurer::apply_filters(stats.to_v2(), config)? {
                Some(filtered) => {
                    stats = FileStats::from_v2(filtered);
                    processed.push(IndexedResult { index, key, entry, stats });
                }
                None => {
                    // フィルタ条件を満たさない場合は結果にもキャッシュにも残さない
                }
            }
        } else {
            pending.push((index, key, entry));
        }
    }

    if !pending.is_empty() {
        let measure_input: Vec<FileEntry> = pending.iter().map(|(_, _, entry)| entry.clone()).collect();
        let measured = measure_all(measure_input, config)?;
        let mut measured_map: HashMap<PathBuf, FileStats> =
            measured.into_iter().map(|stat| (stat.path.clone(), stat)).collect();

        for (index, key, entry) in pending.into_iter() {
            if let Some(stats) = measured_map.remove(&entry.path) {
                processed.push(IndexedResult { index, key, entry, stats });
                changed_files.push(processed.last().unwrap().entry.path.clone());
            }
        }
    }

    processed.sort_by_key(|r| r.index);

    let mut retain = HashSet::new();
    let mut results = Vec::with_capacity(processed.len());
    for record in processed {
        retain.insert(record.key.clone());
        cache.update(record.key.clone(), &record.entry, &record.stats, config.cache_verify);
        results.push(record.stats);
    }

    let removed_keys = cache.prune_except(&retain);
    let removed_files = removed_keys.into_iter().map(PathBuf::from).collect();
    if let Err(err) = cache.save() {
        eprintln!("[warn] failed to persist cache: {}", err);
    }

    Ok(MeasurementOutcome::new(results, changed_files, removed_files))
}

fn measure_all(entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
    if entries.len() < 10 || config.jobs == 1 {
        return measure_sequential(entries, config);
    }
    measure_parallel(entries, config)
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
        eprintln!("[warn] {} files failed measurement", failure_count);
    }

    Ok(stats)
}

/// 並列処理版
fn measure_parallel(entries: Vec<FileEntry>, config: &Config) -> Result<Vec<FileStats>> {
    let progress = config.progress.then(|| (AtomicUsize::new(0), entries.len()));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build()
        .map_err(|e| InfrastructureError::ThreadPoolCreation { details: e.to_string() })?;

    let results: Vec<_> = pool.install(|| {
        entries
            .par_iter()
            .map(|entry| {
                let result = FileMeasurer::measure(entry, config);

                if let Some((counter, total)) = progress.as_ref() {
                    let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if current % 100 == 0 || current == *total {
                        eprintln!("[{}/{}] Processing...", current, total);
                    }
                }

                (entry.path.clone(), result)
            })
            .collect()
    });

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
                    eprintln!("[warn] measurement failed for {}: {}", path.display(), e);
                }
                failure_count += 1;
            }
        }
    }

    if config.progress && failure_count > 0 {
        eprintln!("[warn] {} files failed measurement", failure_count);
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
        let result = if config.count_newlines_in_chars {
            measure_entire_file(&entry.path, &entry.meta, config)
        } else {
            measure_by_lines(&entry.path, &entry.meta, config)
        };

        result.ok_or_else(|| {
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
        if let Some(words) = stats.words() {
            if !filters.words_range.contains(words.value()) {
                return Ok(None);
            }
        }

        // 式フィルタ
        if let Some(ast) = &filters.filter_ast {
            if !Self::eval_filter(&stats, ast)? {
                return Ok(None);
            }
        }

        Ok(Some(stats))
    }

    fn eval_filter(stats: &FileStatsV2, ast: &evalexpr::Node) -> Result<bool> {
        use evalexpr::{ContextWithMutableVariables, Value};

        let mut ctx = evalexpr::HashMapContext::new();

        ctx.set_value("lines".into(), Value::Int(stats.lines().value() as i64)).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
        })?;

        ctx.set_value("chars".into(), Value::Int(stats.chars().value() as i64)).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
        })?;

        let words_val = stats.words().map(|w| w.value()).unwrap_or(0);
        ctx.set_value("words".into(), Value::Int(words_val as i64)).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
        })?;

        ctx.set_value("size".into(), Value::Int(stats.size().bytes() as i64)).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
        })?;

        ctx.set_value("ext".into(), Value::String(stats.ext().as_str().to_string())).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
        })?;

        ctx.set_value("name".into(), Value::String(stats.name().as_str().to_string())).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
        })?;

        if let Some(mtime) = stats.mtime() {
            ctx.set_value("mtime".into(), Value::Int(mtime.timestamp().timestamp())).map_err(|e| {
                DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }
            })?;
        }

        ast.eval_boolean_with_context(&ctx).map_err(|e| {
            DomainError::InvalidFilterExpression { expression: String::new(), details: e.to_string() }.into()
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;
    use crate::infrastructure::measurement::strategies::{measure_by_lines, measure_entire_file};

    struct TempFile {
        path: std::path::PathBuf,
    }

    impl TempFile {
        fn new(content: &str) -> Self {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let path = std::env::temp_dir().join(format!("test_{}.txt", unique));
            fs::write(&path, content).unwrap();
            Self { path }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn make_meta(path: &Path) -> FileMeta {
        let size = fs::metadata(path).unwrap().len();
        FileMeta { size, mtime: None, is_text: true, ext: "txt".to_string(), name: "test.txt".to_string() }
    }

    fn make_config() -> Config {
        use crate::domain::{config::Filters, options::OutputFormat};

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
    fn measure_entire_file_handles_newlines() {
        let file = TempFile::new("a\nb\n");
        let mut config = make_config();
        config.count_newlines_in_chars = true;

        let stats = measure_entire_file(&file.path, &make_meta(&file.path), &config).unwrap();

        assert_eq!(stats.lines().value(), 2);
        assert_eq!(stats.chars().value(), 4); // 'a' + '\n' + 'b' + '\n'
    }

    #[test]
    fn text_only_filters_binary() {
        let file = TempFile::new("text\0binary");
        let mut config = make_config();
        config.text_only = true;
        config.count_newlines_in_chars = true;

        let result = measure_entire_file(&file.path, &make_meta(&file.path), &config);
        assert!(result.is_none());
    }

    #[test]
    fn sequential_measurement() {
        let file1 = TempFile::new("content1");
        let file2 = TempFile::new("content2\ncontent3");

        let entries = vec![
            FileEntry { path: file1.path.clone(), meta: make_meta(&file1.path) },
            FileEntry { path: file2.path.clone(), meta: make_meta(&file2.path) },
        ];

        let config = make_config();
        let stats = measure_sequential(entries, &config).unwrap();

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].lines, 1);
        assert_eq!(stats[1].lines, 2);
    }

    #[test]
    fn parallel_measurement() {
        let file1 = TempFile::new("line1\nline2");
        let file2 = TempFile::new("line3\nline4\nline5");

        let entries = vec![
            FileEntry { path: file1.path.clone(), meta: make_meta(&file1.path) },
            FileEntry { path: file2.path.clone(), meta: make_meta(&file2.path) },
        ];

        let mut config = make_config();
        config.jobs = 2;

        let stats = measure_parallel(entries, &config).unwrap();

        assert_eq!(stats.len(), 2);
        // Order may vary in parallel execution
        let total_lines: usize = stats.iter().map(|s| s.lines).sum();
        assert_eq!(total_lines, 5);
    }
}
