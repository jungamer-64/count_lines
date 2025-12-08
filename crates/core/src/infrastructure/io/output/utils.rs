// src/infrastructure/io/output/utils.rs
use std::path::Path;

use crate::{
    domain::{config::Config, model::FileStats},
    shared::path::logical_absolute,
};

pub(crate) fn limited<'a>(stats: &'a [FileStats], config: &Config) -> &'a [FileStats] {
    let limit = config.top_n.unwrap_or(stats.len()).min(stats.len());
    &stats[..limit]
}

pub(crate) fn format_ratio(val: usize, total: usize) -> String {
    if total == 0 { "0.0".into() } else { format!("{:.1}", (val as f64) * 100.0 / (total as f64)) }
}

pub(crate) fn format_path(stats: &FileStats, config: &Config) -> String {
    format_entry_path(&stats.path, config.abs_path, config.abs_canonical, config.trim_root.as_deref())
}

pub(crate) fn escape_field(s: &str, sep: char) -> String {
    if sep == ',' {
        // Only quote/escape if necessary (contains separator, quote or newline)
        if s.contains(sep) || s.contains('"') || s.contains('\n') || s.contains('\r') {
            let escaped = s.replace('"', "\\\"");
            format!("\"{escaped}\"")
        } else {
            s.to_string()
        }
    } else {
        s.to_string()
    }
}

pub(crate) fn truncate_rows<T>(rows: &mut Vec<T>, limit: Option<usize>) {
    if let Some(n) = limit {
        rows.truncate(n);
    }
}

pub(crate) fn safe_key_label(key: &str) -> String {
    key.replace('|', "\\|")
}

fn format_entry_path(path: &Path, abs_path: bool, abs_canonical: bool, trim_root: Option<&Path>) -> String {
    // Resolve the path according to configuration flags.
    let mut path_buf = if abs_path {
        if abs_canonical {
            path.canonicalize().unwrap_or_else(|_| logical_absolute(path))
        } else {
            logical_absolute(path)
        }
    } else {
        path.to_path_buf()
    };

    // If a root prefix should be trimmed, resolve the root similarly and strip it.
    if let Some(root) = trim_root {
        let root_buf = if abs_path {
            if abs_canonical {
                root.canonicalize().unwrap_or_else(|_| root.to_path_buf())
            } else {
                logical_absolute(root)
            }
        } else {
            root.to_path_buf()
        };
        if let Ok(stripped) = path_buf.strip_prefix(&root_buf) {
            path_buf = stripped.to_path_buf();
        }
    }

    path_buf.display().to_string()
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::Duration};

    use tempfile::tempdir;

    use super::*;
    use crate::domain::{
        config::{Config, Filters},
        model::{FileStats, FileStatsBuilder},
        options::{OutputFormat, SortKey, WatchOutput},
        value_objects::{CharCount, FileExtension, FileName, FilePath, FileSize, LineCount},
    };

    fn sample_stats(path: impl Into<PathBuf>) -> FileStats {
        let pathbuf: PathBuf = path.into();
        let ext_str = pathbuf.extension().and_then(|s| s.to_str()).unwrap_or("").to_string();

        FileStatsBuilder::new(FilePath::new(pathbuf.clone()))
            .lines(LineCount::new(10))
            .chars(CharCount::new(100))
            .words(None)
            .size(FileSize::new(256))
            .ext(FileExtension::new(ext_str.into()))
            .name(FileName::new("sample.rs".into()))
            .build()
            .to_legacy()
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
    fn limited_respects_top_n() {
        let stats = vec![sample_stats("a.rs"), sample_stats("b.rs"), sample_stats("c.rs")];
        let mut config = base_config();
        config.top_n = Some(2);

        let limited_slice = limited(&stats, &config);
        assert_eq!(limited_slice.len(), 2);
        assert_eq!(limited_slice[0].path, stats[0].path);
        assert_eq!(limited_slice[1].path, stats[1].path);
    }

    #[test]
    fn limited_uses_full_slice_when_no_limit() {
        let stats = vec![sample_stats("a.rs"), sample_stats("b.rs")];
        let config = base_config();
        let limited_slice = limited(&stats, &config);
        assert_eq!(limited_slice.len(), stats.len());
    }

    #[test]
    fn format_ratio_handles_zero_total() {
        assert_eq!(format_ratio(5, 0), "0.0");
    }

    #[test]
    fn format_ratio_formats_percentage() {
        assert_eq!(format_ratio(1, 4), "25.0");
    }

    #[test]
    fn escape_field_quotes_and_escapes_for_csv() {
        let escaped = escape_field("a\"b,c", ',');
        assert_eq!(escaped, "\"a\"\"b,c\"");
    }

    #[test]
    fn escape_field_leaves_tsv_untainted() {
        let input = "a\tb";
        assert_eq!(escape_field(input, '\t'), input);
    }

    #[test]
    fn truncate_rows_limits_vector_length() {
        let mut rows = vec![1, 2, 3, 4];
        truncate_rows(&mut rows, Some(2));
        assert_eq!(rows, vec![1, 2]);
    }

    #[test]
    fn truncate_rows_with_none_keeps_all() {
        let mut rows = vec![1, 2, 3];
        truncate_rows(&mut rows, None);
        assert_eq!(rows, vec![1, 2, 3]);
    }

    #[test]
    fn safe_key_label_escapes_pipes() {
        assert_eq!(safe_key_label("foo|bar|baz"), "foo\\|bar\\|baz");
    }

    #[test]
    fn format_path_respects_relative_paths() {
        let stats = sample_stats(PathBuf::from("src/lib.rs"));
        let config = base_config();
        assert_eq!(format_path(&stats, &config), "src/lib.rs");
    }

    #[test]
    fn format_path_with_abs_path_uses_logical_absolute() {
        let mut config = base_config();
        config.abs_path = true;
        let stats = sample_stats(PathBuf::from("src/lib.rs"));
        let expected = crate::shared::path::logical_absolute(stats.path.as_path()).display().to_string();
        assert_eq!(format_path(&stats, &config), expected);
    }

    #[test]
    fn format_path_with_canonical_option_resolves_dot_segments() {
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("file.rs");
        std::fs::write(&file_path, "fn main() {}").expect("write file");

        let path_with_dot = dir.path().join(".").join("file.rs");
        let stats = sample_stats(path_with_dot.clone());

        let mut config = base_config();
        config.abs_path = true;
        config.abs_canonical = true;

        let formatted = format_path(&stats, &config);
        assert_eq!(formatted, file_path.canonicalize().unwrap().display().to_string());
    }

    #[test]
    fn format_path_trims_root_prefix() {
        let dir = tempdir().expect("temp dir");
        let root = dir.path();
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested");
        let file_path = nested.join("file.rs");
        std::fs::write(&file_path, "fn main() {}").expect("write file");

        let stats = sample_stats(file_path.clone());
        let mut config = base_config();
        config.abs_path = true;
        config.trim_root = Some(root.to_path_buf());

        let formatted = format_path(&stats, &config);
        let expected = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.clone())
            .strip_prefix(root)
            .unwrap()
            .display()
            .to_string();
        assert_eq!(formatted, expected);
    }
}
