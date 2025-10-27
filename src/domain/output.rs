mod delimited;
mod jsonl;
mod markdown;
mod structured;
mod table;
mod utils;
mod writer;

use crate::domain::config::Config;
use crate::domain::options::OutputFormat;
use crate::foundation::types::FileStats;

/// Emit results to the configured output format.
pub fn emit(stats: &[FileStats], config: &Config) -> anyhow::Result<()> {
    let mut writer = writer::OutputWriter::create(config)?;
    match config.format {
        OutputFormat::Json => structured::output_json(stats, config, &mut writer),
        OutputFormat::Yaml => structured::output_yaml(stats, config, &mut writer),
        OutputFormat::Csv => delimited::output_delimited(stats, config, ',', &mut writer),
        OutputFormat::Tsv => delimited::output_delimited(stats, config, '\t', &mut writer),
        OutputFormat::Table => table::output_table(stats, config, &mut writer),
        OutputFormat::Md => markdown::output_markdown(stats, config, &mut writer),
        OutputFormat::Jsonl => jsonl::output_jsonl(stats, config, &mut writer),
    }
}
