use clap::{ValueEnum, builder::PossibleValue};

use crate::domain::options::{OutputFormat, WatchOutput};

impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            OutputFormat::Table,
            OutputFormat::Csv,
            OutputFormat::Tsv,
            OutputFormat::Json,
            OutputFormat::Yaml,
            OutputFormat::Md,
            OutputFormat::Jsonl,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let value = match self {
            OutputFormat::Table => PossibleValue::new("table"),
            OutputFormat::Csv => PossibleValue::new("csv"),
            OutputFormat::Tsv => PossibleValue::new("tsv"),
            OutputFormat::Json => PossibleValue::new("json"),
            OutputFormat::Yaml => PossibleValue::new("yaml"),
            OutputFormat::Md => PossibleValue::new("md"),
            OutputFormat::Jsonl => PossibleValue::new("jsonl"),
        };
        Some(value)
    }
}

impl ValueEnum for WatchOutput {
    fn value_variants<'a>() -> &'a [Self] {
        &[WatchOutput::Full, WatchOutput::Jsonl]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let value = match self {
            WatchOutput::Full => PossibleValue::new("full"),
            WatchOutput::Jsonl => PossibleValue::new("jsonl"),
        };
        Some(value)
    }
}
