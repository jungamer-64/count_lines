use clap::ValueEnum;
use count_lines_core::domain::options::{OutputFormat, WatchOutput};

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum CliOutputFormat {
    Table,
    Csv,
    Tsv,
    Json,
    Yaml,
    Md,
    Jsonl,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(value: CliOutputFormat) -> Self {
        match value {
            CliOutputFormat::Table => OutputFormat::Table,
            CliOutputFormat::Csv => OutputFormat::Csv,
            CliOutputFormat::Tsv => OutputFormat::Tsv,
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::Yaml => OutputFormat::Yaml,
            CliOutputFormat::Md => OutputFormat::Md,
            CliOutputFormat::Jsonl => OutputFormat::Jsonl,
        }
    }
}

impl From<OutputFormat> for CliOutputFormat {
    fn from(value: OutputFormat) -> Self {
        match value {
            OutputFormat::Table => CliOutputFormat::Table,
            OutputFormat::Csv => CliOutputFormat::Csv,
            OutputFormat::Tsv => CliOutputFormat::Tsv,
            OutputFormat::Json => CliOutputFormat::Json,
            OutputFormat::Yaml => CliOutputFormat::Yaml,
            OutputFormat::Md => CliOutputFormat::Md,
            OutputFormat::Jsonl => CliOutputFormat::Jsonl,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum CliWatchOutput {
    Full,
    Jsonl,
}

impl From<CliWatchOutput> for WatchOutput {
    fn from(value: CliWatchOutput) -> Self {
        match value {
            CliWatchOutput::Full => WatchOutput::Full,
            CliWatchOutput::Jsonl => WatchOutput::Jsonl,
        }
    }
}

impl From<WatchOutput> for CliWatchOutput {
    fn from(value: WatchOutput) -> Self {
        match value {
            WatchOutput::Full => CliWatchOutput::Full,
            WatchOutput::Jsonl => CliWatchOutput::Jsonl,
        }
    }
}
