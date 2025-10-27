// src/compute.rs
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

/// Process all discovered file entries and return their computed statistics.
pub fn process_entries(config: &Config) -> anyhow::Result<Vec<FileStats>> {
    let entries = crate::files::collect_entries(config)?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build()?;
    let stats = pool.install(|| {
        entries
            .par_iter()
            .filter_map(|e| FileMeasurer::measure(e, config))
            .collect()
    });
    Ok(stats)
}

/// Apply sorting to file statistics in-place based on configuration.
pub fn apply_sort(stats: &mut [FileStats], config: &Config) {
    if config.total_only || config.summary_only || config.sort_specs.is_empty() {
        return;
    }
    for (key, desc) in config.sort_specs.iter().rev() {
        stats.sort_by(|a, b| {
            let ord = Sorter::compare(a, b, *key);
            if *desc {
                ord.reverse()
            } else {
                ord
            }
        });
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
        if config.text_only && !entry.meta.is_text {
            return None;
        }
        let stats = if config.count_newlines_in_chars {
            Self::measure_whole(&entry.path, &entry.meta, config)?
        } else {
            Self::measure_by_lines(&entry.path, &entry.meta, config)?
        };
        Self::apply_filters(stats, config)
    }
    fn measure_whole(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
        let mut file = File::open(path).ok()?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).ok()?;
        if config.text_only && buf.contains(&0) {
            return None;
        }
        let content = String::from_utf8_lossy(&buf);
        let bytes = content.as_bytes();
        let newline_count = bytecount::count(bytes, b'\n');
        let lines = if bytes.is_empty() {
            0
        } else if bytes.last() == Some(&b'\n') {
            newline_count
        } else {
            newline_count + 1
        };
        let chars = content.chars().count();
        let words = config.words.then(|| content.split_whitespace().count());
        Some(FileStats::new(
            path.to_path_buf(),
            lines,
            chars,
            words,
            meta,
        ))
    }
    fn measure_by_lines(path: &Path, meta: &FileMeta, config: &Config) -> Option<FileStats> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        let (mut lines, mut chars, mut words) = (0, 0, 0);
        let mut line = String::new();
        loop {
            line.clear();
            let n = reader.read_line(&mut line).ok()?;
            if n == 0 {
                break;
            }
            if line.ends_with('\n') {
                line.pop();
                if line.ends_with('\r') {
                    line.pop();
                }
            }
            lines += 1;
            chars += line.chars().count();
            if config.words {
                words += line.split_whitespace().count();
            }
        }
        Some(FileStats::new(
            path.to_path_buf(),
            lines,
            chars,
            config.words.then_some(words),
            meta,
        ))
    }
    fn apply_filters(stats: FileStats, config: &Config) -> Option<FileStats> {
        if !config.filters.lines_range.contains(stats.lines) {
            return None;
        }
        if !config.filters.chars_range.contains(stats.chars) {
            return None;
        }
        if !config
            .filters
            .words_range
            .contains(stats.words.unwrap_or(0))
        {
            return None;
        }
        if let Some(ast) = &config.filters.filter_ast {
            if !Self::eval_filter(&stats, ast)? {
                return None;
            }
        }
        Some(stats)
    }
    fn eval_filter(stats: &FileStats, ast: &evalexpr::Node) -> Option<bool> {
        let mut ctx = evalexpr::HashMapContext::new();
        ctx.set_value("lines".into(), Value::Int(stats.lines as i64))
            .ok()?;
        ctx.set_value("chars".into(), Value::Int(stats.chars as i64))
            .ok()?;
        ctx.set_value(
            "words".into(),
            Value::Int(stats.words.unwrap_or(0) as i64),
        )
        .ok()?;
        ctx.set_value("size".into(), Value::Int(stats.size as i64))
            .ok()?;
        ctx.set_value("ext".into(), Value::String(stats.ext.clone()))
            .ok()?;
        ctx.set_value("name".into(), Value::String(stats.name.clone()))
            .ok()?;
        if let Some(mt) = stats.mtime {
            ctx.set_value("mtime".into(), Value::Int(mt.timestamp()))
                .ok()?;
        }
        ast.eval_boolean_with_context(&ctx).ok()
    }
}

/// Aggregation results for a single grouping key.
#[derive(Debug, Clone)]
pub struct AggregationGroup {
    pub key: String,
    pub lines: usize,
    pub chars: usize,
    pub count: usize,
}

impl AggregationGroup {
    fn new(key: String, lines: usize, chars: usize, count: usize) -> Self {
        Self {
            key,
            lines,
            chars,
            count,
        }
    }
}

/// Aggregator to group and summarise file statistics by requested keys.
pub struct Aggregator;
impl Aggregator {
    pub fn aggregate(stats: &[FileStats], by_keys: &[ByKey]) -> Vec<(String, Vec<AggregationGroup>)> {
        by_keys
            .iter()
            .map(|key| Self::aggregate_by_key(stats, key))
            .collect()
    }
    fn aggregate_by_key(stats: &[FileStats], key: &ByKey) -> (String, Vec<AggregationGroup>) {
        match key {
            ByKey::Ext => Self::aggregate_by_ext(stats),
            ByKey::Dir(depth) => Self::aggregate_by_dir(stats, *depth),
            ByKey::Mtime(gran) => Self::aggregate_by_mtime(stats, *gran),
        }
    }
    fn aggregate_by_ext(stats: &[FileStats]) -> (String, Vec<AggregationGroup>) {
        let map = Self::build_aggregation_map(stats, |s| {
            if s.ext.is_empty() {
                "(noext)".to_string()
            } else {
                s.ext.clone()
            }
        });
        ("By Extension".to_string(), Self::map_to_sorted_groups(map))
    }
    fn aggregate_by_dir(stats: &[FileStats], depth: usize) -> (String, Vec<AggregationGroup>) {
        let map = Self::build_aggregation_map(stats, |s| crate::util::get_dir_key(&s.path, depth));
        (
            format!("By Directory (depth={depth})"),
            Self::map_to_sorted_groups(map),
        )
    }
    fn aggregate_by_mtime(stats: &[FileStats], gran: crate::cli::Granularity) -> (String, Vec<AggregationGroup>) {
        let map = Self::build_aggregation_map(stats, |s| {
            s.mtime
                .map(|mt| crate::util::mtime_bucket(mt, gran))
                .unwrap_or_else(|| "(no mtime)".to_string())
        });
        let gran_label = match gran {
            crate::cli::Granularity::Day => "day",
            crate::cli::Granularity::Week => "week",
            crate::cli::Granularity::Month => "month",
        };
        (
            format!("By Mtime ({gran_label})"),
            Self::map_to_sorted_groups(map),
        )
    }
    fn build_aggregation_map<F>(
        stats: &[FileStats],
        key_fn: F,
    ) -> HashMap<String, (usize, usize, usize)>
    where
        F: Fn(&FileStats) -> String,
    {
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
    fn map_to_sorted_groups(
        map: HashMap<String, (usize, usize, usize)>,
    ) -> Vec<AggregationGroup> {
        let mut groups: Vec<AggregationGroup> = map
            .into_iter()
            .map(|(key, (lines, chars, count))| AggregationGroup::new(key, lines, chars, count))
            .collect();
        groups.sort_by(|a, b| b.lines.cmp(&a.lines));
        groups
    }
}