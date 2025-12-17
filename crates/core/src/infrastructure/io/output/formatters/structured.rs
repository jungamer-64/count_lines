// crates/core/src/infrastructure/io/output/formatters/structured.rs
use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    error::Result,
    infrastructure::{
        io::output::utils::format_path,
        serialization::{JsonFile, JsonGroup, JsonGroupRow, JsonOutput, JsonSummary},
    },
};

pub fn output_json(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
    let output = build_json_output(stats, config);
    serde_json::to_writer_pretty(&mut *out, &output)?;
    writeln!(out)?;
    Ok(())
}

#[cfg(feature = "yaml")]
pub fn output_yaml(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
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
    JsonOutput {
        version: crate::VERSION,
        files,
        summary,
        by,
    }
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

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use chrono::TimeZone;
    use serde_json::Value;

    use super::*;
    use crate::domain::{
        config::{ByKey, Config, Filters},
        model::{FileStats, FileStatsBuilder},
        options::{OutputFormat, SortKey, WatchOutput},
        value_objects::{
            CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime,
            WordCount,
        },
    };

    fn sample_stats(
        path: impl Into<PathBuf>,
        lines: usize,
        chars: usize,
        words: Option<usize>,
        mtime: Option<chrono::DateTime<chrono::Local>>,
    ) -> FileStats {
        let pathbuf: PathBuf = path.into();
        let ext_str = pathbuf
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let mut builder = FileStatsBuilder::new(FilePath::new(pathbuf.clone()))
            .lines(LineCount::new(lines))
            .chars(CharCount::new(chars))
            .size(FileSize::new((chars * 2) as u64))
            .ext(FileExtension::new(ext_str.into()))
            .name(FileName::new("file.rs".into()));
        if let Some(w) = words {
            builder = builder.words(Some(WordCount::new(w)));
        }
        if let Some(m) = mtime {
            builder = builder.mtime(Some(ModificationTime::new(m)));
        }
        builder.build().to_legacy()
    }

    fn base_config() -> Config {
        Config {
            format: OutputFormat::Json,
            sort_specs: vec![(SortKey::Lines, true)],
            top_n: None,
            by_modes: vec![],
            summary_only: false,
            total_only: false,
            by_limit: None,
            filters: Filters::default(),
            hidden: false,
            follow: false,
            use_git: false,
            case_insensitive_dedup: false,
            respect_gitignore: true,
            use_ignore_overrides: false,
            jobs: 1,
            no_default_prune: false,
            max_depth: None,
            enumerator_threads: None,
            abs_path: false,
            abs_canonical: false,
            trim_root: None,
            words: false,
            sloc: false,
            count_newlines_in_chars: false,
            text_only: false,
            fast_text_detect: false,
            files_from: None,
            files_from0: None,
            paths: vec![PathBuf::from(".")],
            mtime_since: None,
            mtime_until: None,
            total_row: false,
            progress: false,
            ratio: false,
            output: None,
            strict: false,
            incremental: false,
            cache_dir: None,
            cache_verify: false,
            clear_cache: false,
            watch: false,
            watch_interval: Duration::from_secs(1),
            watch_output: WatchOutput::Full,
            compare: None,
        }
    }

    #[test]
    fn json_output_contains_files_and_summary() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None, None)];
        let config = base_config();
        let mut buffer = Vec::new();

        output_json(&stats, &config, &mut buffer).expect("json output succeeds");
        let json_str = String::from_utf8(buffer).expect("utf8");
        let value: Value = serde_json::from_str(&json_str).expect("parse json");

        assert_eq!(value["version"], crate::VERSION);
        assert_eq!(value["files"].as_array().unwrap().len(), 1);
        assert_eq!(value["files"][0]["file"], "src/lib.rs");
        assert_eq!(value["summary"]["lines"], 10);
        assert!(
            value["summary"]["words"].is_null(),
            "words should be omitted when disabled"
        );
        assert!(
            value.get("by").is_none(),
            "by should be absent when no aggregations requested"
        );
    }

    #[test]
    fn json_output_includes_words_and_groups() {
        let stats = vec![
            sample_stats("src/lib.rs", 8, 80, Some(4), None),
            sample_stats("README", 2, 20, Some(1), None),
        ];
        let mut config = base_config();
        config.words = true;
        config.by_modes = vec![ByKey::Ext];
        config.by_limit = Some(1);

        let mut buffer = Vec::new();
        output_json(&stats, &config, &mut buffer).expect("json output succeeds");
        let json_str = String::from_utf8(buffer).expect("utf8");
        let value: Value = serde_json::from_str(&json_str).expect("parse json");

        assert_eq!(value["summary"]["words"], 5);
        let groups = value["by"].as_array().expect("groups present");
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0]["label"], "By Extension");
        let rows = groups[0]["rows"].as_array().expect("rows present");
        assert_eq!(rows.len(), 1, "by_limit should truncate rows");
        assert_eq!(rows[0]["key"], "rs");
    }

    #[test]
    fn json_output_includes_mtime_when_present() {
        let mtime = chrono::Local
            .with_ymd_and_hms(2024, 5, 1, 12, 0, 0)
            .single()
            .expect("valid local datetime");
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None, Some(mtime))];
        let config = base_config();
        let mut buffer = Vec::new();

        output_json(&stats, &config, &mut buffer).expect("json output succeeds");
        let json_str = String::from_utf8(buffer).expect("utf8");
        let value: Value = serde_json::from_str(&json_str).expect("parse json");

        assert_eq!(
            value["files"][0]["mtime"],
            serde_json::json!(mtime.to_rfc3339())
        );
    }

    #[cfg(feature = "yaml")]
    #[test]
    fn yaml_output_serializes_expected_structure() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None, None)];
        let config = base_config();
        let mut buffer = Vec::new();

        output_yaml(&stats, &config, &mut buffer).expect("yaml output succeeds");
        let yaml = String::from_utf8(buffer).expect("utf8");

        assert!(yaml.contains("version:"));
        assert!(yaml.contains("files:"));
        assert!(yaml.contains("summary:"));
    }
}
