use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    infrastructure::io::output::utils::{format_path, format_ratio, limited, safe_key_label, truncate_rows},
};

pub fn output_markdown(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    write_markdown_header(config, out)?;
    write_markdown_rows(stats, config, out)?;
    write_markdown_aggregations(stats, config, out)?;
    Ok(())
}

fn write_markdown_header(config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
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

fn write_markdown_rows(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    let summary = Summary::from_stats(stats);
    for s in limited(stats, config) {
        let path = format_path(s, config).replace('|', "\\|");
        if config.words {
            if config.ratio {
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
            } else {
                writeln!(out, "| {} | {} | {} | {} |", s.lines, s.chars, s.words.unwrap_or(0), path)?;
            }
        } else if config.ratio {
            writeln!(
                out,
                "| {} | {} | {} | {} | {} |",
                format_ratio(s.lines, summary.lines),
                s.lines,
                format_ratio(s.chars, summary.chars),
                s.chars,
                path
            )?;
        } else {
            writeln!(out, "| {} | {} | {} |", s.lines, s.chars, path)?;
        }
    }
    Ok(())
}

fn write_markdown_aggregations(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
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
