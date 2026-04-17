// crates/cli/src/options.rs
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "lowercase")]
pub enum OutputFormat {
    Table,
    Csv,
    Tsv,
    Json,
    Yaml,
    Md,
    Jsonl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[value(rename_all = "lowercase")]
pub enum WatchOutput {
    Full,
    Jsonl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    Lines,
    Chars,
    Words,
    Size,
    Name,
    Ext,
    /// SLOC (Source Lines of Code)
    Sloc,
}

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

        Ok(Self(specs))
    }
}

fn parse_single_spec(part: &str) -> Result<(SortKey, bool), String> {
    let (key_str, desc) = part.split_once(':').map_or((part, false), |(k, d)| {
        (k.trim(), matches!(d.trim(), "desc" | "DESC"))
    });

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
        "sloc" => Ok(SortKey::Sloc),
        other => Err(format!("Unknown sort key: {other}")),
    }
}
