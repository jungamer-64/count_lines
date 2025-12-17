pub mod formatters;
mod utils;
mod writer;

#[cfg(feature = "yaml")]
use formatters::output_yaml;
use formatters::{output_delimited, output_json, output_jsonl, output_markdown, output_table};

#[cfg(not(feature = "yaml"))]
use crate::error::InfrastructureError;
use crate::{
    domain::{config::Config, model::FileStats, options::OutputFormat},
    error::Result,
};

/// Emit results to the configured output format.
pub fn emit(stats: &[FileStats], config: &Config) -> Result<()> {
    let mut writer = writer::OutputWriter::create(config)?;
    match config.format {
        OutputFormat::Json => output_json(stats, config, &mut writer),
        OutputFormat::Yaml => {
            #[cfg(feature = "yaml")]
            {
                output_yaml(stats, config, &mut writer)
            }

            #[cfg(not(feature = "yaml"))]
            {
                Err(
                    InfrastructureError::OutputError("YAML support disabled at build time".into())
                        .into(),
                )
            }
        }
        OutputFormat::Csv => output_delimited(stats, config, ',', &mut writer),
        OutputFormat::Tsv => output_delimited(stats, config, '\t', &mut writer),
        OutputFormat::Table => output_table(stats, config, &mut writer),
        OutputFormat::Md => output_markdown(stats, config, &mut writer),
        OutputFormat::Jsonl => output_jsonl(stats, config, &mut writer),
    }
}
