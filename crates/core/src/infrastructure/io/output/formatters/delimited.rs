use crate::domain::config::Config;
use crate::domain::model::{FileStats, Summary};
use crate::infrastructure::io::output::utils::{escape_field, format_path, limited};
use std::io::Write;

pub fn output_delimited(
    stats: &[FileStats],
    config: &Config,
    sep: char,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    write_delimited_header(config, sep, out)?;
    write_delimited_rows(stats, config, sep, out)?;
    if config.total_row {
        write_delimited_total(stats, config, sep, out)?;
    }
    Ok(())
}

fn write_delimited_header(config: &Config, sep: char, out: &mut impl Write) -> anyhow::Result<()> {
    if config.words {
        writeln!(out, "lines{sep}chars{sep}words{sep}file")?;
    } else {
        writeln!(out, "lines{sep}chars{sep}file")?;
    }
    Ok(())
}

fn write_delimited_rows(
    stats: &[FileStats],
    config: &Config,
    sep: char,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    for s in limited(stats, config) {
        let path = escape_field(&format_path(s, config), sep);
        if config.words {
            writeln!(
                out,
                "{}{sep}{}{sep}{}{sep}{}",
                s.lines,
                s.chars,
                s.words.unwrap_or(0),
                path
            )?;
        } else {
            writeln!(out, "{}{sep}{}{sep}{}", s.lines, s.chars, path)?;
        }
    }
    Ok(())
}

fn write_delimited_total(
    stats: &[FileStats],
    config: &Config,
    sep: char,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let summary = Summary::from_stats(stats);
    let total_label = escape_field("TOTAL", sep);
    if config.words {
        writeln!(
            out,
            "{}{sep}{}{sep}{}{sep}{}",
            summary.lines, summary.chars, summary.words, total_label
        )?;
    } else {
        writeln!(
            out,
            "{}{sep}{}{sep}{}",
            summary.lines, summary.chars, total_label
        )?;
    }
    Ok(())
}
