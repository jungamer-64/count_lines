use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    infrastructure::{
        io::output::utils::format_path,
        serialization::{JsonFile, JsonGroup, JsonGroupRow, JsonOutput, JsonSummary},
    },
};

pub fn output_json(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    let output = build_json_output(stats, config);
    serde_json::to_writer_pretty(&mut *out, &output)?;
    writeln!(out)?;
    Ok(())
}

pub fn output_yaml(stats: &[FileStats], config: &Config, out: &mut impl Write) -> anyhow::Result<()> {
    let output = build_json_output(stats, config);
    let yaml_str = serde_yaml::to_string(&output)?;
    writeln!(out, "{}", yaml_str)?;
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
    JsonOutput { version: crate::VERSION, files, summary, by }
}

fn build_json_groups(stats: &[FileStats], config: &Config) -> Option<Vec<JsonGroup>> {
    let groups = Aggregator::aggregate(stats, &config.by_modes);
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
                .map(|g| JsonGroupRow { key: g.key, lines: g.lines, chars: g.chars, count: g.count })
                .collect();
            JsonGroup { label, rows: json_rows }
        })
        .collect();
    Some(json_groups)
}
