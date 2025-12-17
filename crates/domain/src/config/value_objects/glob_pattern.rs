use std::path::Path;

use globset::{Glob, GlobMatcher};

/// Wrapper around `globset` matchers that supports both path and name checks.
#[derive(Debug, Clone)]
pub struct GlobPattern {
    original: String,
    matcher: GlobMatcher,
}

impl GlobPattern {
    pub fn new(pattern: &str) -> Result<Self, globset::Error> {
        let glob = Glob::new(pattern)?;
        let matcher = glob.compile_matcher();
        Ok(Self {
            original: pattern.to_string(),
            matcher,
        })
    }

    pub fn matches(&self, value: &str) -> bool {
        self.matcher.is_match(value)
    }

    pub fn matches_path(&self, path: &Path) -> bool {
        self.matcher.is_match(path)
    }

    pub fn pattern(&self) -> &str {
        &self.original
    }
}
