use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    error::Result,
    infrastructure::io::output::utils::{format_path, format_ratio, limited, safe_key_label, truncate_rows},
};

pub fn output_markdown(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
    write_markdown_header(config, out)?;
    write_markdown_rows(stats, config, out)?;
    write_markdown_aggregations(stats, config, out)?;
    Ok(())
}

fn write_markdown_header(config: &Config, out: &mut impl Write) -> Result<()> {
    if config.words {
        if config.ratio {
            writeln!(
                out,
                "| LINES% | LINES | CHARS% | CHARS | WORDS | FILE |\n|---:|---:|---:|---:|---:|:---|"
            )?;
        } else {
            writeln!(out, "| LINES | CHARS | WORDS | FILE |\n|---:|---:|---:|:---|")?;
        }
    } else if config.ratio {
        writeln!(out, "| LINES% | LINES | CHARS% | CHARS | FILE |\n|---:|---:|---:|---:|:---|")?;
    } else {
        writeln!(out, "| LINES | CHARS | FILE |\n|---:|---:|:---|")?;
    }
    Ok(())
}

fn write_markdown_rows(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
    let summary = Summary::from_stats(stats);
    for s in limited(stats, config) {
        let path = format_path(s, config).replace('|', "\\|");
        match (config.words, config.ratio) {
            (true, true) => write_row_words_ratio(out, s, &summary, &path)?,
            (true, false) => write_row_words(out, s, &path)?,
            (false, true) => write_row_ratio(out, s, &summary, &path)?,
            (false, false) => write_row_basic(out, s, &path)?,
        }
    }
    Ok(())
}

fn write_row_words_ratio(out: &mut impl Write, s: &FileStats, summary: &Summary, path: &str) -> Result<()> {
    writeln!(
        out,
        "| {} | {} | {} | {} | {} | {} |",
        format_ratio(s.lines, summary.lines),
        s.lines,
        format_ratio(s.chars, summary.chars),
        s.chars,
        s.words.unwrap_or(0),
        path
    )?;
    Ok(())
}

fn write_row_words(out: &mut impl Write, s: &FileStats, path: &str) -> Result<()> {
    writeln!(out, "| {} | {} | {} | {} |", s.lines, s.chars, s.words.unwrap_or(0), path)?;
    Ok(())
}

fn write_row_ratio(out: &mut impl Write, s: &FileStats, summary: &Summary, path: &str) -> Result<()> {
    writeln!(
        out,
        "| {} | {} | {} | {} | {} |",
        format_ratio(s.lines, summary.lines),
        s.lines,
        format_ratio(s.chars, summary.chars),
        s.chars,
        path
    )?;
    Ok(())
}

fn write_row_basic(out: &mut impl Write, s: &FileStats, path: &str) -> Result<()> {
    writeln!(out, "| {} | {} | {} |", s.lines, s.chars, path)?;
    Ok(())
}

fn write_markdown_aggregations(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
    let groups = Aggregator::aggregate(stats, &config.by_modes);
    for (label, mut rows) in groups {
        writeln!(out, "\n### {label}\n")?;
        writeln!(out, "| LINES | CHARS | KEY | COUNT |\n|---:|---:|:---|---:|")?;
        truncate_rows(&mut rows, config.by_limit);
        for g in rows {
            let key = safe_key_label(&g.key);
            writeln!(out, "| {} | {} | {} | {} |", g.lines, g.chars, key, g.count)?;
        }
    }
    Ok(())
}
