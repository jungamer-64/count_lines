use evalexpr::{ContextWithMutableVariables, Value};
use rayon::prelude::*;

use crate::{
    domain::{
        config::Config,
        model::{FileEntry, FileStats},
    },
    infrastructure::measurement::strategies::{measure_by_lines, measure_entire_file},
};

/// Process all discovered file entries and return their computed statistics.
pub fn measure_entries(entries: Vec<FileEntry>, config: &Config) -> anyhow::Result<Vec<FileStats>> {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(config.jobs).build()?;
    let stats =
        pool.install(|| entries.par_iter().filter_map(|e| FileMeasurer::measure(e, config)).collect());
    Ok(stats)
}

struct FileMeasurer;

impl FileMeasurer {
    fn measure(entry: &FileEntry, config: &Config) -> Option<FileStats> {
        if config.text_only && !entry.meta.is_text {
            return None;
        }
        let stats = if config.count_newlines_in_chars {
            measure_entire_file(&entry.path, &entry.meta, config)?
        } else {
            measure_by_lines(&entry.path, &entry.meta, config)?
        };
        Self::apply_filters(stats, config)
    }

    fn apply_filters(stats: FileStats, config: &Config) -> Option<FileStats> {
        if !config.filters.lines_range.contains(stats.lines) {
            return None;
        }
        if !config.filters.chars_range.contains(stats.chars) {
            return None;
        }
        if !config.filters.words_range.contains(stats.words.unwrap_or(0)) {
            return None;
        }
        if let Some(ast) = &config.filters.filter_ast
            && !Self::eval_filter(&stats, ast)?
        {
            return None;
        }
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
        if let Some(mt) = stats.mtime {
            ctx.set_value("mtime".into(), Value::Int(mt.timestamp())).ok()?;
        }
        ast.eval_boolean_with_context(&ctx).ok()
    }
}
