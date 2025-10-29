// src/domain/options.rs
use std::str::FromStr;

/// Output format options for the tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Table,
    Csv,
    Tsv,
    Json,
    Yaml,
    Md,
    Jsonl,
}

/// Sorting keys available for ordering results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Lines,
    Chars,
    Words,
    Size,
    Name,
    Ext,
}

/// Sort specification. Example: `lines:desc,chars:desc,name`.
#[derive(Debug, Clone)]
pub struct SortSpec(pub Vec<(SortKey, bool)>);

impl FromStr for SortSpec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let specs = s
            .split(',')
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .map(parse_single_spec)
            .collect::<Result<Vec<_>, _>>()?;

        if specs.is_empty() {
            return Err("empty sort spec".into());
        }
        Ok(SortSpec(specs))
    }
}

fn parse_single_spec(part: &str) -> Result<(SortKey, bool), String> {
    let (key_str, desc) =
        part.split_once(':').map_or((part, false), |(k, d)| (k.trim(), matches!(d.trim(), "desc" | "DESC")));

    let key = parse_sort_key(key_str)?;
    Ok((key, desc))
}

fn parse_sort_key(key_str: &str) -> Result<SortKey, String> {
    match key_str.to_ascii_lowercase().as_str() {
        "lines" => Ok(SortKey::Lines),
        "chars" => Ok(SortKey::Chars),
        "words" => Ok(SortKey::Words),
        "size" => Ok(SortKey::Size),
        "name" => Ok(SortKey::Name),
        "ext" => Ok(SortKey::Ext),
        other => Err(format!("Unknown sort key: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_size_sort_key() {
        let spec: SortSpec = "size:desc".parse().expect("size sort parses");
        assert!(matches!(spec.0.as_slice(), [(SortKey::Size, true)]));
    }

    #[test]
    fn rejects_unknown_sort_key() {
        let err = "invalid".parse::<SortSpec>().expect_err("invalid key should fail");
        assert!(err.contains("Unknown sort key"));
    }
}
