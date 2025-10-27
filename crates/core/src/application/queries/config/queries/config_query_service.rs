use std::{collections::HashSet, path::PathBuf};

use anyhow::{Result, anyhow};
use evalexpr::Node;

use super::super::commands::{ConfigOptions, FilterOptions};
use crate::{
    domain::{
        config::{ByKey, Config, Filters, Range, SizeRange},
        grouping::ByMode,
        options::SortKey,
    },
    shared::{path::logical_absolute, patterns::parse_patterns},
};

/// Query service responsible for materialising configuration read models.
pub struct ConfigQueryService;

impl ConfigQueryService {
    /// Build a [`Config`] read model from a [`ConfigOptions`] query.
    ///
    /// # Errors
    /// Propagates validation and parsing failures while materialising the configuration.
    pub fn build(query: ConfigOptions) -> Result<Config> {
        let ConfigOptions {
            format,
            sort_specs,
            top_n,
            by,
            summary_only,
            total_only,
            by_limit,
            filters,
            hidden,
            follow,
            use_git,
            jobs,
            no_default_prune,
            abs_path,
            abs_canonical,
            trim_root,
            words,
            count_newlines_in_chars,
            text_only,
            fast_text_detect,
            files_from,
            files_from0,
            paths,
            mtime_since,
            mtime_until,
            total_row,
            progress,
            ratio,
            output,
            strict,
            compare,
        } = query;

        let filters = build_filters(&filters)?;
        let jobs = jobs.unwrap_or_else(num_cpus::get).max(1);
        let words = compute_words_flag(words, &sort_specs, &filters);
        let paths = normalise_paths(paths);
        let by_modes = convert_by_modes(by);
        let abs_path = abs_path || abs_canonical;

        Ok(Config {
            format,
            sort_specs,
            top_n,
            by_modes,
            summary_only,
            total_only,
            by_limit,
            filters,
            hidden,
            follow,
            use_git,
            jobs,
            no_default_prune,
            abs_path,
            abs_canonical,
            trim_root: trim_root.map(|p| logical_absolute(&p)),
            words,
            count_newlines_in_chars,
            text_only,
            fast_text_detect,
            files_from,
            files_from0,
            paths,
            mtime_since,
            mtime_until,
            total_row,
            progress,
            ratio,
            output,
            strict,
            compare,
        })
    }
}

fn normalise_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    if paths.is_empty() { vec![PathBuf::from(".")] } else { paths }
}

fn build_filters(options: &FilterOptions) -> Result<Filters> {
    let filter_ast = options
        .filter
        .as_ref()
        .map(|expr| evalexpr::build_operator_tree(expr).map_err(|e| anyhow!(e)))
        .transpose()?;

    Ok(Filters {
        include_patterns: parse_patterns(&options.include)?,
        exclude_patterns: parse_patterns(&options.exclude)?,
        include_paths: parse_patterns(&options.include_path)?,
        exclude_paths: parse_patterns(&options.exclude_path)?,
        exclude_dirs: parse_patterns(&options.exclude_dir)?,
        ext_filters: parse_extensions(options.ext.as_deref()),
        size_range: SizeRange::new(options.min_size, options.max_size),
        lines_range: Range::new(options.min_lines, options.max_lines),
        chars_range: Range::new(options.min_chars, options.max_chars),
        words_range: Range::new(options.min_words, options.max_words),
        filter_ast,
    })
}

fn parse_extensions(ext_arg: Option<&str>) -> HashSet<String> {
    ext_arg.map(|s| s.split(',').map(|e| e.trim().to_lowercase()).collect()).unwrap_or_default()
}

fn compute_words_flag(explicit_words: bool, sort_specs: &[(SortKey, bool)], filters: &Filters) -> bool {
    let filter_depends_on_words = filters.filter_ast.as_ref().is_some_and(filter_expr_requires_words);

    explicit_words
        || filters.words_range.min.is_some()
        || filters.words_range.max.is_some()
        || filter_depends_on_words
        || sort_specs.iter().any(|(key, _)| matches!(key, SortKey::Words))
}

fn filter_expr_requires_words(ast: &Node) -> bool {
    ast.iter_variable_identifiers().any(|ident| ident == "words")
}

fn convert_by_modes(by: Vec<ByMode>) -> Vec<ByKey> {
    by.into_iter().filter(|b| !matches!(b, ByMode::None)).map(convert_by_mode).collect()
}

fn convert_by_mode(mode: ByMode) -> ByKey {
    match mode {
        ByMode::Ext => ByKey::Ext,
        ByMode::Dir(d) => ByKey::Dir(d),
        ByMode::Mtime(g) => ByKey::Mtime(g),
        ByMode::None => unreachable!(),
    }
}
