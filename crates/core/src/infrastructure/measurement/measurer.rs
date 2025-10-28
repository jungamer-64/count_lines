// crates/core/src/infrastructure/measurement/measurer.rs
use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;

use crate::{
    domain::{
        config::Config,
        error::InfrastructureError,
        model::{FileEntry, FileStats},
    },
    infrastructure::measurement::strategies::{measure_by_lines, measure_entire_file},
};

pub fn measure_entries(
    entries: Vec<FileEntry>,
    config: &Config,
) -> Result<Vec<FileStats>, InfrastructureError> {
    let progress = config.progress.then(|| {
        let counter = AtomicUsize::new(0);
        let total = entries.len();
        (counter, total)
    });
    
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build()
        .map_err(|e| InfrastructureError::ThreadPoolCreation(e.to_string()))?;
    
    let results: Vec<_> = pool.install(|| {
        entries
            .par_iter()
            .filter_map(|entry| {
                let result = FileMeasurer::measure(entry, config);
                
                if let Some((counter, total)) = progress.as_ref() {
                    let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if current % 100 == 0 || current == *total {
                        eprintln!("[{current}/{total}] Processing...");
                    }
                }
                
                result
            })
            .collect()
    });
    
    Ok(results)
}

struct FileMeasurer;

impl FileMeasurer {
    fn measure(entry: &FileEntry, config: &Config) -> Option<FileStats> {
        if config.text_only && !entry.meta.is_text {
            return None;
        }
        
        let stats = Self::compute_stats(entry, config)?;
        Self::apply_filters(stats, config)
    }
    
    fn compute_stats(entry: &FileEntry, config: &Config) -> Option<FileStats> {
        if config.count_newlines_in_chars {
            measure_entire_file(&entry.path, &entry.meta, config)
        } else {
            measure_by_lines(&entry.path, &entry.meta, config)
        }
    }
    
    fn apply_filters(stats: FileStats, config: &Config) -> Option<FileStats> {
        let filters = &config.filters;
        
        if !filters.lines_range.contains(stats.lines) {
            return None;
        }
        
        if !filters.chars_range.contains(stats.chars) {
            return None;
        }
        
        if !filters.words_range.contains(stats.words.unwrap_or(0)) {
            return None;
        }
        
        if let Some(ast) = &filters.filter_ast {
            if !Self::eval_filter(&stats, ast)? {
                return None;
            }
        }
        
        Some(stats)
    }
    
    fn eval_filter(stats: &FileStats, ast: &evalexpr::Node) -> Option<bool> {
        use evalexpr::{ContextWithMutableVariables, Value};
        
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
