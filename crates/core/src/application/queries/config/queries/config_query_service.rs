use std::collections::HashSet;

use crate::{
    application::queries::config::commands::{ConfigOptions, FilterOptions},
    domain::{
        config::{ByKey, Config, Filters, Range, SizeRange},
        grouping::ByMode,
    },
    error::{DomainError, Result},
    shared::{path::logical_absolute, patterns::parse_patterns},
};

/// 設定クエリサービス
pub struct ConfigQueryService;

impl ConfigQueryService {
    /// 設定を構築
    pub fn build(query: ConfigOptions) -> Result<Config> {
        let filters = Self::build_filters(&query.filters)?;
        let jobs = query.jobs.unwrap_or_else(num_cpus::get).max(1);
        let words = Self::should_enable_words(&query, &filters);
        let paths = Self::normalize_paths(query.paths);
        let by_modes = Self::convert_by_modes(query.by);
        let abs_path = query.abs_path || query.abs_canonical;
        let trim_root = query.trim_root.map(|p| logical_absolute(&p));

        Ok(Config {
            format: query.format,
            sort_specs: query.sort_specs,
            top_n: query.top_n,
            by_modes,
            summary_only: query.summary_only,
            total_only: query.total_only,
            by_limit: query.by_limit,
            filters,
            hidden: query.hidden,
            follow: query.follow,
            use_git: query.use_git,
            jobs,
            no_default_prune: query.no_default_prune,
            abs_path,
            abs_canonical: query.abs_canonical,
            trim_root,
            words,
            count_newlines_in_chars: query.count_newlines_in_chars,
            text_only: query.text_only,
            fast_text_detect: query.fast_text_detect,
            files_from: query.files_from,
            files_from0: query.files_from0,
            paths,
            mtime_since: query.mtime_since,
            mtime_until: query.mtime_until,
            total_row: query.total_row,
            progress: query.progress,
            ratio: query.ratio,
            output: query.output,
            strict: query.strict,
            compare: query.compare,
        })
    }

    fn build_filters(options: &FilterOptions) -> Result<Filters> {
        let filter_ast = options
            .filter
            .as_ref()
            .map(|expr| {
                evalexpr::build_operator_tree(expr).map_err(|e| DomainError::InvalidFilterExpression {
                    expression: expr.clone(),
                    details: e.to_string(),
                })
            })
            .transpose()?;

        Ok(Filters {
            include_patterns: parse_patterns(&options.include)?,
            exclude_patterns: parse_patterns(&options.exclude)?,
            include_paths: parse_patterns(&options.include_path)?,
            exclude_paths: parse_patterns(&options.exclude_path)?,
            exclude_dirs: parse_patterns(&options.exclude_dir)?,
            ext_filters: Self::parse_extensions(options.ext.as_deref()),
            size_range: SizeRange::new(options.min_size, options.max_size),
            lines_range: Range::new(options.min_lines, options.max_lines),
            chars_range: Range::new(options.min_chars, options.max_chars),
            words_range: Range::new(options.min_words, options.max_words),
            filter_ast,
        })
    }

    fn parse_extensions(ext_arg: Option<&str>) -> HashSet<String> {
        ext_arg
            .map(|s| {
                s.split(',').map(str::trim).filter(|e| !e.is_empty()).map(|e| e.to_lowercase()).collect()
            })
            .unwrap_or_default()
    }

    fn should_enable_words(query: &ConfigOptions, filters: &Filters) -> bool {
        query.words
            || filters.words_range.min.is_some()
            || filters.words_range.max.is_some()
            || Self::filter_uses_words(filters)
            || Self::sort_uses_words(&query.sort_specs)
    }

    fn filter_uses_words(filters: &Filters) -> bool {
        filters
            .filter_ast
            .as_ref()
            .map(|ast| ast.iter_variable_identifiers().any(|id| id == "words"))
            .unwrap_or(false)
    }

    fn sort_uses_words(sort_specs: &[(crate::domain::options::SortKey, bool)]) -> bool {
        sort_specs.iter().any(|(key, _)| matches!(key, crate::domain::options::SortKey::Words))
    }

    fn normalize_paths(paths: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        if paths.is_empty() { vec![std::path::PathBuf::from(".")] } else { paths }
    }

    fn convert_by_modes(modes: Vec<ByMode>) -> Vec<ByKey> {
        modes.into_iter().filter(|mode| !matches!(mode, ByMode::None)).map(Self::convert_by_mode).collect()
    }

    fn convert_by_mode(mode: ByMode) -> ByKey {
        match mode {
            ByMode::Ext => ByKey::Ext,
            ByMode::Dir(depth) => ByKey::Dir(depth),
            ByMode::Mtime(granularity) => ByKey::Mtime(granularity),
            ByMode::None => unreachable!("None modes should be filtered out"),
        }
    }
}
