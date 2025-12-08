// crates/core/src/infrastructure/io/output/formatters/jsonl.rs
use std::io::Write;

use crate::{
    domain::{
        config::Config,
        model::{FileStats, Summary},
    },
    error::Result,
    infrastructure::io::output::utils::format_path,
};

pub fn output_jsonl(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use chrono::TimeZone;
    use serde_json::Value;

    use super::*;
    use crate::domain::{
        config::{Config, Filters},
        model::{FileStats, FileStatsBuilder},
        options::{OutputFormat, SortKey, WatchOutput},
        value_objects::{
            CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, ModificationTime, WordCount,
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
        let ext_str = pathbuf.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

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
            format: OutputFormat::Jsonl,
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
    fn jsonl_outputs_file_entries_and_total() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None, None)];
        let config = base_config();
        let mut buffer = Vec::new();

        output_jsonl(&stats, &config, &mut buffer).expect("jsonl output succeeds");
        let json_string = String::from_utf8(buffer).expect("utf8");
        let lines: Vec<_> = json_string.lines().collect();

        assert_eq!(lines.len(), 2, "should emit one line per file plus summary");
        let file_item: Value = serde_json::from_str(lines[0]).expect("parse file line");
        assert_eq!(file_item["type"], "file");
        assert_eq!(file_item["file"], "src/lib.rs");
        let total_item: Value = serde_json::from_str(lines[1]).expect("parse total line");
        assert_eq!(total_item["type"], "total");
        assert!(total_item["words"].is_null(), "words should be null when disabled");
    }

    #[test]
    fn jsonl_includes_words_when_enabled() {
        let stats = vec![
            sample_stats("src/lib.rs", 8, 80, Some(4), None),
            sample_stats("src/main.rs", 2, 20, Some(1), None),
        ];
        let mut config = base_config();
        config.words = true;

        let mut buffer = Vec::new();
        output_jsonl(&stats, &config, &mut buffer).expect("jsonl output succeeds");
        let json_string = String::from_utf8(buffer).expect("utf8");
        let lines: Vec<_> = json_string.lines().collect();

        assert_eq!(lines.len(), 3);
        let total_item: Value = serde_json::from_str(lines[2]).expect("parse total line");
        assert_eq!(total_item["words"], 5);
    }

    #[test]
    fn jsonl_includes_mtime_when_available() {
        let mtime =
            chrono::Local.with_ymd_and_hms(2024, 5, 1, 12, 34, 56).single().expect("valid local datetime");
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None, Some(mtime))];
        let config = base_config();

        let mut buffer = Vec::new();
        output_jsonl(&stats, &config, &mut buffer).expect("jsonl output succeeds");
        let json_string = String::from_utf8(buffer).expect("utf8");
        let lines: Vec<_> = json_string.lines().collect();
        let file_item: Value = serde_json::from_str(lines[0]).expect("parse file line");

        assert_eq!(file_item["mtime"], serde_json::json!(mtime.to_rfc3339()));
    }
}
