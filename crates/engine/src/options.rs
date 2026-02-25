// crates/engine/src/options.rs
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    Table,
    Csv,
    Tsv,
    Json,
    Yaml,
    Md,
    Jsonl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchOutput {
    Full,
    Jsonl,
}

/// Output mode (alternative to `summary_only`/`total_only`)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputMode {
    /// Show all files individually
    #[default]
    Full,
    /// Summary by extension/directory only
    Summary,
    /// Show total only
    TotalOnly,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Granularity {
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByMode {
    Ext,
    Dir(usize),
    Mtime(Granularity),
}

impl FromStr for ByMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ext" => Ok(Self::Ext),
            _ if s.starts_with("dir") => {
                let depth = s
                    .strip_prefix("dir=")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(1);
                Ok(Self::Dir(depth))
            }
            _ if s.starts_with("mtime") => {
                let gran = s.split(':').nth(1).unwrap_or("day");
                let g = match gran {
                    "day" => Granularity::Day,
                    "week" => Granularity::Week,
                    "month" => Granularity::Month,
                    _ => return Err(format!("Unknown mtime granularity: {gran}")),
                };
                Ok(Self::Mtime(g))
            }
            other => Err(format!("Unknown --by mode: {other}")),
        }
    }
}
