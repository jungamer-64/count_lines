use std::path::Path;

use chrono::{DateTime, Local};

use crate::domain::config::{Config, Filters};

/// デフォルトで除外するディレクトリ
const DEFAULT_PRUNE_DIRS: &[&str] = &[
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".venv",
    "venv",
    "build",
    "dist",
    "target",
    ".cache",
    ".direnv",
    ".mypy_cache",
    ".pytest_cache",
    "coverage",
    "__pycache__",
    ".idea",
    ".next",
    ".nuxt",
];

/// ディレクトリエントリを処理すべきか判定
pub(crate) fn should_process_entry(entry: &walkdir::DirEntry, config: &Config) -> bool {
    let path = entry.path();

    // 隠しファイル/ディレクトリのチェック
    if !config.hidden && is_hidden(path) {
        return false;
    }

    // ディレクトリの場合の特別処理
    if entry.file_type().is_dir() {
        return !should_prune_directory(entry, config);
    }

    true
}

/// ディレクトリを除外すべきか判定
fn should_prune_directory(entry: &walkdir::DirEntry, config: &Config) -> bool {
    if !config.no_default_prune {
        let dir_name = entry.file_name().to_string_lossy();
        if DEFAULT_PRUNE_DIRS.contains(&dir_name.as_ref()) {
            return true;
        }
    }

    let path = entry.path();
    config.filters.exclude_dirs.iter().any(|pattern| pattern.matches_path(path))
}

/// パスマッチャー
pub(crate) struct PathMatcher;

impl PathMatcher {
    /// パスがフィルタ条件に一致するか判定
    pub(crate) fn matches(path: &Path, config: &Config) -> bool {
        Self::matches_name_patterns(path, &config.filters)
            && Self::matches_path_patterns(path, &config.filters)
            && Self::matches_extension(path, &config.filters)
            && Self::matches_metadata(path, config)
    }

    fn matches_name_patterns(path: &Path, filters: &Filters) -> bool {
        let Some(file_name) = path.file_name() else {
            return false;
        };
        let name = file_name.to_string_lossy();

        // includeパターンのチェック
        if !filters.include_patterns.is_empty()
            && !filters.include_patterns.iter().any(|p| p.matches(&name)) {
                return false;
            }

        // excludeパターンのチェック
        !filters.exclude_patterns.iter().any(|p| p.matches(&name))
    }

    fn matches_path_patterns(path: &Path, filters: &Filters) -> bool {
        // includeパスパターンのチェック
        if !filters.include_paths.is_empty()
            && !filters.include_paths.iter().any(|p| p.matches_path(path)) {
                return false;
            }

        // excludeパスパターンのチェック
        !filters.exclude_paths.iter().any(|p| p.matches_path(path))
    }

    fn matches_extension(path: &Path, filters: &Filters) -> bool {
        if filters.ext_filters.is_empty() {
            return true;
        }

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| filters.ext_filters.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }

    fn matches_metadata(path: &Path, config: &Config) -> bool {
        let Ok(metadata) = std::fs::metadata(path) else {
            return true; // メタデータ取得失敗時は除外しない
        };

        // サイズチェック
        if !config.filters.size_range.contains(metadata.len()) {
            return false;
        }

        // 更新時刻チェック
        Self::matches_mtime(&metadata, config)
    }

    fn matches_mtime(metadata: &std::fs::Metadata, config: &Config) -> bool {
        let Ok(modified_sys) = metadata.modified() else {
            return true;
        };

        let modified: DateTime<Local> = modified_sys.into();

        if let Some(since) = config.mtime_since
            && modified < since {
                return false;
            }

        if let Some(until) = config.mtime_until
            && modified > until {
                return false;
            }

        true
    }
}

/// パスが隠しファイルか判定
fn is_hidden(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()).map(|name| name.starts_with('.')).unwrap_or(false)
}
