use crate::foundation::util;
use anyhow::{anyhow, Result};
use evalexpr::Node;
use std::collections::HashSet;

use super::GlobPattern;

/// Input options for building [`Filters`].
#[derive(Debug, Default)]
pub struct FilterOptions {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub include_path: Vec<String>,
    pub exclude_path: Vec<String>,
    pub exclude_dir: Vec<String>,
    pub ext: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub min_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub min_chars: Option<usize>,
    pub max_chars: Option<usize>,
    pub min_words: Option<usize>,
    pub max_words: Option<usize>,
    pub filter: Option<String>,
}

/// Filtering parameters derived from structured options.
#[derive(Debug, Default)]
pub struct Filters {
    pub include_patterns: Vec<GlobPattern>,
    pub exclude_patterns: Vec<GlobPattern>,
    pub include_paths: Vec<GlobPattern>,
    pub exclude_paths: Vec<GlobPattern>,
    pub exclude_dirs: Vec<GlobPattern>,
    pub ext_filters: HashSet<String>,
    pub size_range: SizeRange,
    pub lines_range: Range,
    pub chars_range: Range,
    pub words_range: Range,
    pub filter_ast: Option<Node>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SizeRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Range {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl Range {
    fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, v: usize) -> bool {
        self.min.map_or(true, |m| v >= m) && self.max.map_or(true, |x| v <= x)
    }
}

impl SizeRange {
    fn new(min: Option<u64>, max: Option<u64>) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, v: u64) -> bool {
        self.min.map_or(true, |m| v >= m) && self.max.map_or(true, |x| v <= x)
    }
}

impl Filters {
    /// Construct a `Filters` from structured filter options. This parses patterns
    /// and builds an optional evalexpr AST for advanced filtering.
    pub fn from_options(options: &FilterOptions) -> Result<Self> {
        let filter_ast = options
            .filter
            .as_ref()
            .map(|expr| evalexpr::build_operator_tree(expr).map_err(|e| anyhow!(e)))
            .transpose()?;

        Ok(Self {
            include_patterns: util::parse_patterns(&options.include)?,
            exclude_patterns: util::parse_patterns(&options.exclude)?,
            include_paths: util::parse_patterns(&options.include_path)?,
            exclude_paths: util::parse_patterns(&options.exclude_path)?,
            exclude_dirs: util::parse_patterns(&options.exclude_dir)?,
            ext_filters: parse_extensions(options.ext.as_deref()),
            size_range: SizeRange::new(options.min_size, options.max_size),
            lines_range: Range::new(options.min_lines, options.max_lines),
            chars_range: Range::new(options.min_chars, options.max_chars),
            words_range: Range::new(options.min_words, options.max_words),
            filter_ast,
        })
    }
}

fn parse_extensions(ext_arg: Option<&str>) -> HashSet<String> {
    ext_arg
        .map(|s| {
            s.split(',')
                .map(|e| e.trim().to_lowercase())
                .collect()
        })
        .unwrap_or_default()
}
