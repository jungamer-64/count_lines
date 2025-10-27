use crate::domain::config::Config;
use crate::foundation::types::{FileEntry, FileMeta, FileStats};
use evalexpr::{ContextWithMutableVariables, Value};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Process all discovered file entries and return their computed statistics.
pub fn process_entries(config: &Config) -> anyhow::Result<Vec<FileStats>> {
    let entries = crate::domain::files::collect_entries(config)?;
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
