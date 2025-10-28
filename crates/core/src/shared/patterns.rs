use crate::domain::error::DomainError;
use anyhow::Result;
use glob::Pattern;

/// glob パターンのコレクションをパースする
pub fn parse_patterns(patterns: &[String]) -> Result<Vec<Pattern>> {
    patterns
        .iter()
        .map(|pattern| {
            Pattern::new(pattern).map_err(|source| DomainError::InvalidPattern {
                pattern: pattern.clone(),
                source,
            })
        })
        .collect::<std::result::Result<Vec<_>, DomainError>>()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_patterns() {
        let patterns = vec!["*.rs".to_string(), "src/**".to_string()];
        let result = parse_patterns(&patterns);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn rejects_invalid_patterns() {
        let patterns = vec!["[[".to_string()];
        assert!(parse_patterns(&patterns).is_err());
    }
}
