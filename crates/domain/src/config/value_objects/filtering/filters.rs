// crates/domain/src/config/value_objects/filtering/filters.rs
use std::collections::HashSet;

use super::{Range, SizeRange};
use crate::config::value_objects::GlobPattern;

#[cfg(feature = "eval")]
pub type FilterAst = evalexpr::Node;

#[cfg(not(feature = "eval"))]
#[derive(Debug, Clone, Default)]
pub struct FilterAst;

/// Filtering parameters derived from structured options.
#[derive(Debug, Default, Clone)]
pub struct Filters {
    pub include_patterns: Vec<GlobPattern>,
    pub exclude_patterns: Vec<GlobPattern>,
    pub include_paths: Vec<GlobPattern>,
    pub exclude_paths: Vec<GlobPattern>,
    pub exclude_dirs: Vec<GlobPattern>,
    pub exclude_dirs_only: Vec<GlobPattern>,
    pub ext_filters: HashSet<String>,
    pub overrides_include: Vec<String>,
    pub overrides_exclude: Vec<String>,
    pub force_text_exts: Vec<String>,
    pub force_binary_exts: Vec<String>,
    pub size_range: SizeRange,
    pub lines_range: Range,
    pub chars_range: Range,
    pub words_range: Range,
    pub filter_ast: Option<FilterAst>,
}
