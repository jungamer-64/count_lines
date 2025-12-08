// src/infrastructure/io/output/writer.rs
use std::io::{BufWriter, Write};

use crate::{
    domain::config::Config,
    error::{InfrastructureError, Result},
    infrastructure::persistence::FileWriter,
};

pub(crate) struct OutputWriter(Box<dyn Write>);

impl OutputWriter {
    pub(crate) fn create(config: &Config) -> Result<Self> {
        let writer: Box<dyn Write> = if let Some(path) = &config.output {
            Box::new(
                FileWriter::create(path)
                    .map_err(|source| InfrastructureError::FileWrite { path: path.clone(), source })?,
            )
        } else {
            Box::new(BufWriter::new(std::io::stdout()))
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

#[cfg(test)]
mod tests {
    use std::{io::Write, path::PathBuf, time::Duration};

    use tempfile::tempdir;

    use super::*;
    use crate::{
        domain::{
            config::{Config, Filters},
            options::{OutputFormat, SortKey, WatchOutput},
        },
        error::{CountLinesError, InfrastructureError},
    };

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
    fn create_writes_to_configured_file() {
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("out.txt");

        let mut config = base_config();
        config.output = Some(file_path.clone());

        let mut writer = OutputWriter::create(&config).expect("writer creates");
        writer.write_all(b"hello world").expect("write succeeds");
        writer.flush().expect("flush succeeds");

        let contents = std::fs::read_to_string(&file_path).expect("read written file");
        assert_eq!(contents, "hello world");
    }

    #[test]
    fn create_propagates_file_errors() {
        let dir = tempdir().expect("temp dir");
        let mut config = base_config();
        config.output = Some(dir.path().to_path_buf());

        match OutputWriter::create(&config) {
            Ok(_) => panic!("creating writer should fail for directory path"),
            Err(CountLinesError::Infrastructure(InfrastructureError::FileWrite { path, .. })) => {
                assert_eq!(path, dir.path());
            }
            Err(other) => panic!("expected FileWrite error, got {other:?}"),
        }
    }
}
