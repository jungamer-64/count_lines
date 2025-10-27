// src/infrastructure/io/output/utils.rs
use std::path::Path;

use crate::{
    domain::{config::Config, model::FileStats},
    shared::path::logical_absolute,
};

pub(crate) fn limited<'a>(stats: &'a [FileStats], config: &Config) -> &'a [FileStats] {
    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    &stats[..limit]
}

pub(crate) fn format_ratio(val: usize, total: usize) -> String {
    if total == 0 { "0.0".into() } else { format!("{:.1}", (val as f64) * 100.0 / (total as f64)) }
}

pub(crate) fn format_path(stats: &FileStats, config: &Config) -> String {
    format_entry_path(&stats.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref())
}

pub(crate) fn escape_field(s: &str, sep: char) -> String {
    if sep == ',' {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

pub(crate) fn truncate_rows<T>(rows: &mut Vec<T>, limit: Option<usize>) {
    if let Some(n) = limit {
        rows.truncate(n);
    }
}

pub(crate) fn safe_key_label(key: &str) -> String {
    key.replace('|', "\\|")
}

fn format_entry_path(path: &Path, abs_path: bool, abs_canonical: bool, trim_root: Option<&Path>) -> String {
    let mut path = if abs_path {
        if abs_canonical {
            path.canonicalize().unwrap_or_else(|_| logical_absolute(path))
        } else {
            logical_absolute(path)
        }
    } else {
        path.to_path_buf()
    };
    if let Some(root) = trim_root
        && let Ok(stripped) = path.strip_prefix(root)
    {
        path = stripped.to_path_buf();
    }
    path.display().to_string()
}
