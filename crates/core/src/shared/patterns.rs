use anyhow::Context as _;

/// Parse a list of glob patterns, returning a vector of compiled patterns or an error.
pub fn parse_patterns(patterns: &[String]) -> anyhow::Result<Vec<glob::Pattern>> {
    patterns.iter().map(|p| glob::Pattern::new(p).with_context(|| format!("Invalid pattern: {p}"))).collect()
}
