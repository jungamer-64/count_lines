// src/output.rs
use std::io::Write;

use crate::config::Config;
use crate::types::{
    FileStats, JsonFile, JsonGroup, JsonGroupRow, JsonOutput, JsonSummary, Summary,
};

/// Emit results to the configured output format.
pub fn emit(stats: &[FileStats], config: &Config) -> anyhow::Result<()> {
    let mut writer = OutputWriter::create(config)?;
    match config.format {
        crate::cli::OutputFormat::Json => output_json(stats, config, &mut writer)?,
        crate::cli::OutputFormat::Yaml => output_yaml(stats, config, &mut writer)?,
        crate::cli::OutputFormat::Csv => {
            output_delimited(stats, config, ',', &mut writer)?
        }
        crate::cli::OutputFormat::Tsv => {
            output_delimited(stats, config, '\t', &mut writer)?
        }
        crate::cli::OutputFormat::Table => output_table(stats, config, &mut writer)?,
        crate::cli::OutputFormat::Md => output_markdown(stats, config, &mut writer)?,
        crate::cli::OutputFormat::Jsonl => output_jsonl(stats, config, &mut writer)?,
    }
    Ok(())
}

struct OutputWriter(Box<dyn Write>);
impl OutputWriter {
    fn create(config: &Config) -> anyhow::Result<Self> {
        let writer: Box<dyn Write> = if let Some(path) = &config.output {
            Box::new(std::io::BufWriter::new(std::fs::File::create(path)?))
        } else {
            Box::new(std::io::BufWriter::new(std::io::stdout()))
        };
        Ok(Self(writer))
    }
}
impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

fn limited<'a>(stats: &'a [FileStats], config: &Config) -> &'a [FileStats] {
    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    &stats[..limit]
}

fn format_ratio(val: usize, total: usize) -> String {
    if total == 0 {
        "0.0".into()
    } else {
        format!("{:.1}", (val as f64) * 100.0 / (total as f64))
    }
}

fn format_path(stats: &FileStats, config: &Config) -> String {
    crate::util::format_path(
        &stats.path,
        config.abs_path,
        config.abs_canonical,
        config.trim_root.as_deref(),
    )
}

fn output_table(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
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
            writeln!(
                out,
                "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\t   WORDS\tFILE"
            )?;
        } else {
            writeln!(out, "    LINES\t CHARACTERS\t   WORDS\tFILE")?;
        }
    } else if config.ratio {
        writeln!(
            out,
            "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\tFILE"
        )?;
    } else {
        writeln!(out, "    LINES\t CHARACTERS\tFILE")?;
    }
    writeln!(out, "----------------------------------------------")?;
    Ok(())
}

fn write_table_rows(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
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
                writeln!(
                    out,
                    "{:>10}\t{:>10}\t{:>7}\t{}",
                    s.lines,
                    s.chars,
                    s.words.unwrap_or(0),
                    path
                )?;
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

fn write_aggregations(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let groups = crate::compute::Aggregator::aggregate(stats, &config.by_modes);
    for (label, mut rows) in groups {
        writeln!(out, "[{label}]")?;
        writeln!(out, "{:>10}\t{:>10}\tKEY", "LINES", "CHARACTERS")?;
        if let Some(n) = config.by_limit {
            rows.truncate(n);
        }
        for g in rows {
            writeln!(
                out,
                "{:>10}\t{:>10}\t{} ({} files)",
                g.lines, g.chars, g.key, g.count
            )?;
        }
        writeln!(out, "---")?;
    }
    Ok(())
}

fn output_summary(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let summary = Summary::from_stats(stats);
    if config.words {
        writeln!(
            out,
            "{:>10}\t{:>10}\t{:>7}\tTOTAL ({} files)\n",
            summary.lines, summary.chars, summary.words, summary.files
        )?;
    } else {
        writeln!(
            out,
            "{:>10}\t{:>10}\tTOTAL ({} files)\n",
            summary.lines, summary.chars, summary.files
        )?;
    }
    Ok(())
}

fn output_delimited(
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

fn write_delimited_header(
    config: &Config,
    sep: char,
    out: &mut impl Write,
) -> anyhow::Result<()> {
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

fn escape_field(s: &str, sep: char) -> String {
    if sep == ',' {
        let escaped = s.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        s.to_string()
    }
}

fn output_markdown(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    write_markdown_header(config, out)?;
    write_markdown_rows(stats, config, out)?;
    write_markdown_aggregations(stats, config, out)?;
    Ok(())
}

fn write_markdown_header(
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    if config.words {
        if config.ratio {
            writeln!(
                out,
                "| LINES% | LINES | CHARS% | CHARS | WORDS | FILE |\n|---:|---:|---:|---:|---:|:---|"
            )?;
        } else {
            writeln!(
                out,
                "| LINES | CHARS | WORDS | FILE |\n|---:|---:|---:|:---|"
            )?;
        }
    } else if config.ratio {
        writeln!(
            out,
            "| LINES% | LINES | CHARS% | CHARS | FILE |\n|---:|---:|---:|---:|:---|"
        )?;
    } else {
        writeln!(out, "| LINES | CHARS | FILE |\n|---:|---:|:---|")?;
    }
    Ok(())
}

fn write_markdown_rows(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
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
                writeln!(
                    out,
                    "| {} | {} | {} | {} |",
                    s.lines, s.chars, s.words.unwrap_or(0), path
                )?;
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
    let groups = crate::compute::Aggregator::aggregate(stats, &config.by_modes);
    for (label, mut rows) in groups {
        writeln!(out, "\n### {label}\n")?;
        writeln!(out, "| LINES | CHARS | KEY | COUNT |\n|---:|---:|:---|---:|")?;
        if let Some(n) = config.by_limit {
            rows.truncate(n);
        }
        for g in rows {
            let key = g.key.replace('|', "\\|");
            writeln!(out, "| {} | {} | {} | {} |", g.lines, g.chars, key, g.count)?;
        }
    }
    Ok(())
}

fn build_json_output(stats: &[FileStats], config: &Config) -> JsonOutput {
    let files = stats
        .iter()
        .map(|s| JsonFile {
            file: format_path(s, config),
            lines: s.lines,
            chars: s.chars,
            words: s.words,
            size: s.size,
            mtime: s.mtime.map(|d| d.to_rfc3339()),
            ext: s.ext.clone(),
        })
        .collect();
    let summary_data = Summary::from_stats(stats);
    let summary = JsonSummary {
        lines: summary_data.lines,
        chars: summary_data.chars,
        words: config.words.then_some(summary_data.words),
        files: summary_data.files,
    };
    let by = build_json_groups(stats, config);
    JsonOutput {
        version: crate::VERSION,
        files,
        summary,
        by,
    }
}

fn build_json_groups(
    stats: &[FileStats],
    config: &Config,
) -> Option<Vec<JsonGroup>> {
    let groups = crate::compute::Aggregator::aggregate(stats, &config.by_modes);
    if groups.is_empty() {
        return None;
    }
    let json_groups = groups
        .into_iter()
        .map(|(label, mut rows)| {
            if let Some(n) = config.by_limit {
                rows.truncate(n);
            }
            let json_rows = rows
                .into_iter()
                .map(|g| JsonGroupRow {
                    key: g.key,
                    lines: g.lines,
                    chars: g.chars,
                    count: g.count,
                })
                .collect();
            JsonGroup {
                label,
                rows: json_rows,
            }
        })
        .collect();
    Some(json_groups)
}

fn output_json(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let output = build_json_output(stats, config);
    serde_json::to_writer_pretty(&mut *out, &output)?;
    writeln!(out)?;
    Ok(())
}

fn output_yaml(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
    let output = build_json_output(stats, config);
    let yaml_str = serde_yaml::to_string(&output)?;
    writeln!(out, "{}", yaml_str)?;
    Ok(())
}

fn output_jsonl(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> anyhow::Result<()> {
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