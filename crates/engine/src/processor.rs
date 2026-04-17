// crates/engine/src/processor.rs
use crate::config::Config;
use crate::error::{EngineError, Result};
use crate::stats::FileStats;
use count_lines_core::config::AnalysisConfig;
use count_lines_core::counter::count_bytes;
use std::path::PathBuf;

pub fn process_file(
    (path, meta): (PathBuf, std::fs::Metadata),
    config: &Config,
) -> Result<FileStats> {
    let mut stats = FileStats::new(path.clone());
    stats.size = meta.len();
    stats.mtime = meta
        .modified()
        .ok()
        .map(chrono::DateTime::<chrono::Local>::from);

    let content = std::fs::read(&path).map_err(|source| EngineError::FileRead {
        path: path.clone(),
        source,
    })?;

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let analysis_config = AnalysisConfig {
        count_words: config.count_words,
        count_sloc: config.count_sloc,
        count_newlines_in_chars: config.count_newlines_in_chars,
        map_ext: config.filter.map_ext.clone(),
    };
    let analysis = count_bytes(&content, extension, &analysis_config);

    stats.lines = analysis.lines;
    stats.chars = analysis.chars;
    stats.words = analysis.words;
    stats.sloc = if config.count_sloc {
        analysis.sloc
    } else {
        None
    };
    stats.is_binary = analysis.is_binary;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_count_chars_trailing_spaces() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        write!(file, "abc \nde \r\nfg ")?;
        let path = file.path().to_path_buf();

        let config = Config {
            count_newlines_in_chars: false,
            filter: crate::config::FilterConfig {
                allow_ext: vec![],
                ..crate::config::FilterConfig::default()
            },
            ..Config::default()
        };

        let meta = std::fs::metadata(&path)?;
        let stats = process_file((path, meta), &config)?;

        assert_eq!(stats.chars, 10);
        assert_eq!(stats.lines, 3);
        Ok(())
    }

    #[test]
    fn test_respects_count_flags() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        write!(file, "one two\nthree\n")?;
        let path = file.path().to_path_buf();
        let meta = std::fs::metadata(&path)?;

        let base_config = Config::default();
        let stats_without_optional = process_file((path.clone(), meta), &base_config)?;
        assert_eq!(stats_without_optional.words, None);
        assert_eq!(stats_without_optional.sloc, None);

        let mut with_optional = base_config;
        with_optional.count_words = true;
        with_optional.count_sloc = true;
        let stats_with_optional =
            process_file((path.clone(), std::fs::metadata(&path)?), &with_optional)?;
        assert_eq!(stats_with_optional.words, Some(3));
        assert!(stats_with_optional.sloc.is_some());
        Ok(())
    }

    #[test]
    fn test_binary_file_marks_binary() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut file = NamedTempFile::new()?;
        file.write_all(&[0_u8, 1_u8, 2_u8, 3_u8])?;
        let path = file.path().to_path_buf();

        let stats = process_file((path.clone(), std::fs::metadata(path)?), &Config::default())?;
        assert!(stats.is_binary);
        assert_eq!(stats.lines, 0);
        Ok(())
    }
}
