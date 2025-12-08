// crates/core/src/infrastructure/io/output/formatters/delimited.rs
use std::io::Write;

use crate::{
    domain::{
        config::Config,
        model::{FileStats, Summary},
    },
    error::Result,
    infrastructure::io::output::utils::{escape_field, format_path, limited},
};

pub fn output_delimited(stats: &[FileStats], config: &Config, sep: char, out: &mut impl Write) -> Result<()> {
    write_delimited_header(config, sep, out)?;
    write_delimited_rows(stats, config, sep, out)?;
    if config.total_row {
        write_delimited_total(stats, config, sep, out)?;
    }
    Ok(())
}

fn write_delimited_header(config: &Config, sep: char, out: &mut impl Write) -> Result<()> {
    if config.words {
        writeln!(out, "lines{sep}chars{sep}words{sep}file")?;
    } else {
        writeln!(out, "lines{sep}chars{sep}file")?;
    }
    Ok(())
}

fn write_delimited_rows(stats: &[FileStats], config: &Config, sep: char, out: &mut impl Write) -> Result<()> {
    for s in limited(stats, config) {
        let path = escape_field(&format_path(s, config), sep);
        if config.words {
            writeln!(out, "{}{sep}{}{sep}{}{sep}{}", s.lines, s.chars, s.words.unwrap_or(0), path)?;
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
) -> Result<()> {
    let summary = Summary::from_stats(stats);
    let total_label = escape_field("TOTAL", sep);
    if config.words {
        writeln!(out, "{}{sep}{}{sep}{}{sep}{}", summary.lines, summary.chars, summary.words, total_label)?;
    } else {
        writeln!(out, "{}{sep}{}{sep}{}", summary.lines, summary.chars, total_label)?;
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
            .name(FileName::new("sample.rs".into()));
        if let Some(w) = words {
            builder = builder.words(Some(WordCount::new(w)));
        }
        builder.build().to_legacy()
    }

    fn base_config() -> Config {
        Config {
            format: OutputFormat::Csv,
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
    fn writes_header_and_rows_without_words() {
        let stats = vec![sample_stats("src/lib.rs", 10, 100, None)];
        let config = base_config();
        let mut buffer = Vec::new();

        output_delimited(&stats, &config, ',', &mut buffer).expect("write succeeds");
        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.starts_with("lines,chars,file"));
        assert!(output.contains("10,100,src/lib.rs"));
        assert!(!output.contains("words"), "words column should be absent");
    }

    #[test]
    fn writes_words_column_and_total_row() {
        let stats =
            vec![sample_stats("src/lib.rs", 10, 100, Some(5)), sample_stats("src/main.rs", 2, 20, Some(1))];
        let mut config = base_config();
        config.words = true;
        config.total_row = true;

        let mut buffer = Vec::new();
        output_delimited(&stats, &config, ',', &mut buffer).expect("write succeeds");
        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.starts_with("lines,chars,words,file"));
        assert!(output.contains("10,100,5,src/lib.rs"));
        assert!(output.contains("2,20,1,src/main.rs"));
        assert!(output.trim_end().ends_with("12,120,6,TOTAL"));
    }

    #[test]
    fn respects_top_n_limit() {
        let stats = vec![
            sample_stats("a.rs", 10, 100, None),
            sample_stats("b.rs", 20, 200, None),
            sample_stats("c.rs", 30, 300, None),
        ];
        let mut config = base_config();
        config.top_n = Some(2);

        let mut buffer = Vec::new();
        output_delimited(&stats, &config, ',', &mut buffer).expect("write succeeds");
        let output = String::from_utf8(buffer).expect("utf8");

        assert!(output.contains("a.rs"));
        assert!(output.contains("b.rs"));
        assert!(!output.contains("c.rs"), "limited output should omit entries beyond top_n");
    }

    #[test]
    fn escapes_fields_for_csv() {
        let stats = vec![sample_stats("src/\"weird\",file.rs", 1, 2, None)];
        let config = base_config();

        let mut buffer = Vec::new();
        output_delimited(&stats, &config, ',', &mut buffer).expect("write succeeds");
        let output = String::from_utf8(buffer).expect("utf8");

        assert!(output.contains("\"src/\"\"weird\"\",file.rs\""));
    }
}
