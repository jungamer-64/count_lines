use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    infrastructure::io::output::utils::{format_path, format_ratio, limited, truncate_rows},
};

pub fn output_table(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    if config.total_only {
        return output_summary(stats, config, out);
    }
    if !config.summary_only {
        write_table_header(config, out)?;
        write_table_rows(stats, config, out)?;
    }
    if !config.total_only {
        write_aggregations(stats, config, out)?;
    }
    output_summary(stats, config, out)
}

fn write_table_header(config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    writeln!(out)?;
    if config.words {
        if config.ratio {
            writeln!(out, "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\t   WORDS\tFILE")?;
        } else {
            writeln!(out, "    LINES\t CHARACTERS\t   WORDS\tFILE")?;
        }
    } else if config.ratio {
        writeln!(out, "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\tFILE")?;
    } else {
        writeln!(out, "    LINES\t CHARACTERS\tFILE")?;
    }
    writeln!(out, "----------------------------------------------")?;
    Ok(())
}

fn write_table_rows(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    let summary = Summary::from_stats(stats);
    for s in limited(stats, config) {
        let path = format_path(s, config);
        if config.words {
            if config.ratio {
                writeln!(
                    out,
                    "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{:>7}\t{}",
                    format_ratio(s.lines, summary.lines),
                    s.lines,
                    format_ratio(s.chars, summary.chars),
                    s.chars,
                    s.words.unwrap_or(0),
                    path
                )?;
            } else {
                writeln!(out, "{:>10}\t{:>10}\t{:>7}\t{}", s.lines, s.chars, s.words.unwrap_or(0), path)?;
            }
        } else if config.ratio {
            writeln!(
                out,
                "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{}",
                format_ratio(s.lines, summary.lines),
                s.lines,
                format_ratio(s.chars, summary.chars),
                s.chars,
                path
            )?;
        } else {
            writeln!(out, "{:>10}\t{:>10}\t{}", s.lines, s.chars, path)?;
        }
    }
    writeln!(out, "---")?;
    Ok(())
}

fn write_aggregations(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    let groups = Aggregator::aggregate(stats, &config.by_modes);
    for (label, mut rows) in groups {
        writeln!(out, "[{label}]")?;
        writeln!(out, "{:>10}\t{:>10}\tKEY", "LINES", "CHARACTERS")?;
        truncate_rows(&mut rows, config.by_limit);
        for g in rows {
            writeln!(out, "{:>10}\t{:>10}\t{} ({} files)", g.lines, g.chars, g.key, g.count)?;
        }
        writeln!(out, "---")?;
    }
    Ok(())
}

fn output_summary(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    let summary = Summary::from_stats(stats);
    if config.words {
        writeln!(
            out,
            "{:>10}\t{:>10}\t{:>7}\tTOTAL ({} files)\n",
            summary.lines, summary.chars, summary.words, summary.files
        )?;
    } else {
        writeln!(out, "{:>10}\t{:>10}\tTOTAL ({} files)\n", summary.lines, summary.chars, summary.files)?;
    }
    Ok(())
}
