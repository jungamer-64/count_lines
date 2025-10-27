use crate::domain::compare;
use crate::domain::compute;
use crate::domain::options::OutputFormat;
use crate::domain::output;
use crate::interface::cli;
use anyhow::{Context, Result};
use atty::Stream;

pub fn run() -> Result<()> {
    let config = cli::load_config()?;

    if let Some((old, new)) = &config.compare {
        let diff = compare::run(old, new).context("compare failed")?;
        println!("{}", diff);
        return Ok(());
    }

    if !matches!(config.format, OutputFormat::Json) && atty::is(Stream::Stdout) {
        eprintln!("count_lines v{} · parallel={}", crate::VERSION, config.jobs);
    }

    if config.progress {
        eprintln!("[count_lines] scanning & measuring...");
    }

    let mut stats = match compute::process_entries(&config) {
        Ok(v) => v,
        Err(e) => {
            if config.strict {
                return Err(e).context("failed to measure entries");
            }
            eprintln!("[warn] {}", e);
            Vec::new()
        }
    };

    compute::apply_sort(&mut stats, &config);
    output::emit(&stats, &config).context("failed to emit output")?;
    Ok(())
}
