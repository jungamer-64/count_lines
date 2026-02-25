// crates/engine/src/options.rs
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported output formats for the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    /// Plain text table format.
    Table,
    /// Comma-separated values.
    Csv,
    /// Tab-separated values.
    Tsv,
    /// JSON array of objects.
    Json,
    /// YAML format.
    Yaml,
    /// Markdown table format.
    Md,
    /// JSON lines format.
    Jsonl,
}

/// Output format specifically for watch mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchOutput {
    /// Full output updated per event.
    Full,
    /// JSON lines output per event.
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

/// Keys to sort the resulting statistics by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortKey {
    /// Sort by number of lines.
    Lines,
    /// Sort by number of characters.
    Chars,
    /// Sort by number of words.
    Words,
    /// Sort by file size in bytes.
    Size,
    /// Sort by alphabetical file name.
    Name,
    /// Sort by file extension.
    Ext,
    /// SLOC (Source Lines of Code)
    Sloc,
}

/// Time granularity for grouping by modification time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Granularity {
    /// Group by day.
    Day,
    /// Group by week.
    Week,
    /// Group by month.
    Month,
}

/// Criteria to group the file statistics by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ByMode {
    /// Group by file extension.
    Ext,
    /// Group by directory up to the given depth.
    Dir(usize),
    /// Group by modification time with given granularity.
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
