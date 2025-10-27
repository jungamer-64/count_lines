// src/domain/output/utils.rs
use crate::domain::config::Config;
use crate::foundation::types::FileStats;
use crate::foundation::util;

pub(crate) fn limited<'a>(stats: &'a [FileStats], config: &Config) -> &'a [FileStats] {
    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    &stats[..limit]
}

pub(crate) fn format_ratio(val: usize, total: usize) -> String {
    if total == 0 {
        "0.0".into()
    } else {
        format!("{:.1}", (val as f64) * 100.0 / (total as f64))
    }
}

pub(crate) fn format_path(stats: &FileStats, config: &Config) -> String {
    util::format_path(
        &stats.path,
        config.abs_path,
        config.abs_canonical,
        config.trim_root.as_deref(),
    )
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
