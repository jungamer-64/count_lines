// crates/core/src/infrastructure/io/output/formatters/table.rs
use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    error::Result,
    infrastructure::io::output::utils::{format_path, format_ratio, limited, truncate_rows},
};

// Header constants moved to module scope to keep header_line small for static analysis
const HEADER_WORDS_RATIO: &str = "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\t   WORDS\tFILE";
const HEADER_WORDS: &str = "    LINES\t CHARACTERS\t   WORDS\tFILE";
const HEADER_RATIO: &str = "    LINES%\t    LINES\t CHARACTERS%\t CHARACTERS\tFILE";
const HEADER_BASIC: &str = "    LINES\t CHARACTERS\tFILE";

pub fn output_table(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
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

fn write_table_header(config: &Config, out: &mut impl Write) -> Result<()> {
    writeln!(out)?;
    writeln!(out, "{}", header_line(config))?;
    writeln!(out, "----------------------------------------------")?;
    Ok(())
}

fn write_table_rows(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
    let summary = Summary::from_stats(stats);
    for s in limited(stats, config) {
        let path = format_path(s, config);
        let line = format_row(s, &summary, config, &path);
        writeln!(out, "{}", line)?;
    }
    writeln!(out, "---")?;
    Ok(())
}

fn header_line(config: &Config) -> &'static str {
    match (config.words, config.ratio) {
        (true, true) => HEADER_WORDS_RATIO,
        (true, false) => HEADER_WORDS,
        (false, true) => HEADER_RATIO,
        (false, false) => HEADER_BASIC,
    }
}

fn format_row(s: &FileStats, summary: &Summary, config: &Config, path: &str) -> String {
    match (config.words, config.ratio) {
        (true, true) => format_row_words_ratio(s, summary, path),
        (true, false) => format_row_words(s, path),
        (false, true) => format_row_ratio(s, summary, path),
        (false, false) => format_row_basic(s, path),
    }
}

fn format_row_words_ratio(s: &FileStats, summary: &Summary, path: &str) -> String {
    format!(
        "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{:>7}\t{}",
        format_ratio(s.lines, summary.lines),
        s.lines,
        format_ratio(s.chars, summary.chars),
        s.chars,
        s.words.unwrap_or(0),
        path
    )
}

fn format_row_words(s: &FileStats, path: &str) -> String {
    format!("{:>10}\t{:>10}\t{:>7}\t{}", s.lines, s.chars, s.words.unwrap_or(0), path)
}

fn format_row_ratio(s: &FileStats, summary: &Summary, path: &str) -> String {
    format!(
        "{:>10}\t{:>10}\t{:>12}\t{:>11}\t{}",
        format_ratio(s.lines, summary.lines),
        s.lines,
        format_ratio(s.chars, summary.chars),
        s.chars,
        path
    )
}

fn format_row_basic(s: &FileStats, path: &str) -> String {
    format!("{:>10}\t{:>10}\t{}", s.lines, s.chars, path)
}

fn write_aggregations(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
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

fn output_summary(stats: &[FileStats], config: &Config, out: &mut impl Write) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use super::*;
    use crate::domain::{
        config::{Config, Filters},
        model::{FileStats, FileStatsBuilder},
        options::{OutputFormat, SortKey, WatchOutput},
        value_objects::{CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, WordCount},
    };

    fn sample_stats(path: impl Into<PathBuf>, lines: usize, chars: usize, words: Option<usize>) -> FileStats {
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
        builder.build().to_legacy()
    }

    fn base_config() -> Config {
        Config {
            format: OutputFormat::Table,
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
    fn table_basic_output_contains_header_and_rows() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None)];
        let config = base_config();

        let mut buffer = Vec::new();
        output_table(&stats, &config, &mut buffer).expect("table output succeeds");
        let output = String::from_utf8(buffer).expect("utf8");

        assert!(output.contains("LINES\t CHARACTERS"), "header should be present");
        assert!(output.contains("src/lib.rs"), "path should be included in output");
        assert!(output.contains("10"), "line count should be formatted");
    }

    #[test]
    fn table_with_words_and_ratio_formats_percentages() {
        let stats =
            vec![sample_stats("src/lib.rs", 8, 80, Some(4)), sample_stats("src/main.rs", 2, 20, Some(1))];
        let mut config = base_config();
        config.words = true;
        config.ratio = true;

        let mut buffer = Vec::new();
        output_table(&stats, &config, &mut buffer).expect("table output succeeds");
        let output = String::from_utf8(buffer).expect("utf8");

        assert!(output.contains("LINES%\t    LINES"), "ratio header should appear");
        assert!(output.contains("80.0"), "line percentage should be formatted");
        assert!(output.contains("src/lib.rs"), "first row path should be present");
        // ensure that the line for src/lib.rs contains a word count
        let has_word_count = output.lines().any(|line| line.contains("src/lib.rs") && line.contains('4'));
        assert!(has_word_count, "word counts should be present");
    }

    #[test]
    fn table_summary_only_outputs_total_without_header() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None)];
        let mut config = base_config();
        config.summary_only = true;

        let mut buffer = Vec::new();
        output_table(&stats, &config, &mut buffer).expect("table output succeeds");
        let output = String::from_utf8(buffer).expect("utf8");

        assert!(!output.contains("LINES%\t"), "header should not be printed when summary_only is set");
        assert!(output.contains("TOTAL (1 files)"), "summary should still be emitted");
    }

    #[test]
    fn table_total_only_skips_rows_and_aggregations() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, Some(2))];
        let mut config = base_config();
        config.total_only = true;
        config.words = true;

        let mut buffer = Vec::new();
        output_table(&stats, &config, &mut buffer).expect("table output succeeds");
        let output = String::from_utf8(buffer).expect("utf8");

        assert!(output.starts_with("        10"), "summary should be first output");
        assert!(!output.contains("src/lib.rs"), "rows should not be printed in total_only mode");
    }

    #[test]
    fn table_writes_aggregations_with_limits() {
        use crate::domain::config::ByKey;

        let stats = vec![
            sample_stats("src/lib.rs", 10, 100, None),
            sample_stats("src/main.rs", 20, 200, None),
            sample_stats("README", 5, 50, None),
        ];

        let mut config = base_config();
        config.by_modes = vec![ByKey::Ext];
        config.by_limit = Some(1);

        let mut buffer = Vec::new();
        output_table(&stats, &config, &mut buffer).expect("table output succeeds");
        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("[By Extension]"), "aggregation header should be present");
        assert!(output.contains("rs (2 files)"), "dominant extension should be listed");
        assert!(!output.contains("(noext)"), "by_limit should truncate to top result");
    }
}
