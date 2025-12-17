// crates/core/src/infrastructure/io/output/formatters/markdown.rs
use std::io::Write;

use crate::{
    domain::{
        analytics::Aggregator,
        config::Config,
        model::{FileStats, Summary},
    },
    error::Result,
    infrastructure::io::output::utils::{
        format_path, format_ratio, limited, safe_key_label, truncate_rows,
    },
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

fn write_row_words_ratio(
    out: &mut impl Write,
    s: &FileStats,
    summary: &Summary,
    path: &str,
) -> Result<()> {
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
    writeln!(
        out,
        "| {} | {} | {} | {} |",
        s.lines,
        s.chars,
        s.words.unwrap_or(0),
        path
    )?;
    Ok(())
}

fn write_row_ratio(
    out: &mut impl Write,
    s: &FileStats,
    summary: &Summary,
    path: &str,
) -> Result<()> {
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

fn write_markdown_aggregations(
    stats: &[FileStats],
    config: &Config,
    out: &mut impl Write,
) -> Result<()> {
    let groups = Aggregator::aggregate(stats, &config.by_modes);
    for (label, mut rows) in groups {
        writeln!(out, "\n### {label}\n")?;
        writeln!(
            out,
            "| LINES | CHARS | KEY | COUNT |\n|---:|---:|:---|---:|"
        )?;
        truncate_rows(&mut rows, config.by_limit);
        for g in rows {
            let key = safe_key_label(&g.key);
            writeln!(out, "| {} | {} | {} | {} |", g.lines, g.chars, key, g.count)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use super::*;
    use crate::domain::{
        config::{ByKey, Config, Filters},
        model::{FileStats, FileStatsBuilder},
        options::{OutputFormat, SortKey, WatchOutput},
        value_objects::{
            CharCount, FileExtension, FileName, FilePath, FileSize, LineCount, WordCount,
        },
    };

    fn sample_stats(
        path: impl Into<PathBuf>,
        lines: usize,
        chars: usize,
        words: Option<usize>,
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
        builder.build().to_legacy()
    }

    fn base_config() -> Config {
        Config {
            format: OutputFormat::Md,
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
    fn markdown_basic_rows_are_rendered() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None)];
        let config = base_config();

        let mut buffer = Vec::new();
        output_markdown(&stats, &config, &mut buffer).expect("markdown output succeeds");
        let markdown = String::from_utf8(buffer).expect("utf8");

        assert!(markdown.contains("| LINES | CHARS | FILE |"));
        assert!(markdown.contains("| 10 | 100 | src/lib.rs |"));
    }

    #[test]
    fn markdown_with_words_and_ratio_formats_columns() {
        let stats = vec![
            sample_stats("src/lib.rs", 8, 80, Some(4)),
            sample_stats("src/main.rs", 2, 20, Some(1)),
        ];
        let mut config = base_config();
        config.words = true;
        config.ratio = true;

        let mut buffer = Vec::new();
        output_markdown(&stats, &config, &mut buffer).expect("markdown output succeeds");
        let markdown = String::from_utf8(buffer).expect("utf8");

        assert!(
            markdown.contains("| LINES% | LINES | CHARS% | CHARS | WORDS | FILE |"),
            "header should reflect words + ratio"
        );
        assert!(markdown.contains("| 80.0 | 8 | 80.0 | 80 | 4 | src/lib.rs |"));
    }

    #[test]
    fn markdown_aggregations_respect_limits_and_escape_keys() {
        let mut stats = vec![
            sample_stats("src/lib.rs", 10, 100, None),
            sample_stats("README", 5, 50, None),
            sample_stats("docs/guide.md", 3, 30, None),
        ];
        let mut special = sample_stats("notes/special.rs", 60, 600, None);
        special.ext = "special|key".into();
        stats.push(special);

        let mut config = base_config();
        config.by_modes = vec![ByKey::Ext];
        config.by_limit = Some(1);

        let mut buffer = Vec::new();
        output_markdown(&stats, &config, &mut buffer).expect("markdown output succeeds");
        let markdown = String::from_utf8(buffer).expect("utf8");
        assert!(
            markdown.contains("### By Extension"),
            "aggregation header should appear"
        );
        assert!(markdown.contains("| LINES | CHARS | KEY | COUNT |"));
        assert!(
            markdown.contains("| 60 | 600 | special\\|key | 1 |"),
            "top extension should be listed"
        );
        // Ensure the aggregation block respects by_limit (only check the aggregation section)
        if let Some(idx) = markdown.find("### By Extension") {
            let aggr = &markdown[idx..];
            assert!(
                !aggr.contains("md"),
                "by_limit should restrict rows in aggregation to the requested number"
            );
        } else {
            panic!("aggregation section not found");
        }
        assert!(
            markdown.contains("special\\|key"),
            "keys with pipe characters should be escaped for markdown tables"
        );
    }
}
