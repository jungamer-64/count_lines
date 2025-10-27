use crate::domain::config::value_objects::GlobPattern;
use evalexpr::Node;
use std::collections::HashSet;

use super::{Range, SizeRange};

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
