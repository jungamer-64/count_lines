use std::io::Write;

use crate::{
    domain::{
        config::Config,
        model::{FileStats, Summary},
    },
    infrastructure::io::output::utils::format_path,
};

pub fn output_jsonl(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    for s in stats {
        let item = serde_json::json!({
            "type": "file",
            "file": format_path(s, config),
            "lines": s.lines,
            "chars": s.chars,
            "words": s.words,
            "size": s.size,
            "mtime": s.mtime.map(|d| d.to_rfc3339()),
            "ext": &s.ext,
        });
        serde_json::to_writer(&mut *out, &item)?;
        writeln!(out)?;
    }
    let summary = Summary::from_stats(stats);
    let total = serde_json::json!({
        "type": "total",
        "version": crate::VERSION,
        "lines": summary.lines,
        "chars": summary.chars,
        "words": if config.words { Some(summary.words) } else { None },
        "files": summary.files,
    });
    serde_json::to_writer(&mut *out, &total)?;
    writeln!(out)?;
    Ok(())
}
