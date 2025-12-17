use clap::Parser;
use count_lines_cli::args::Args;
use count_lines_cli::config::Config;
use count_lines_cli::engine;
use count_lines_cli::presentation;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    let config = Config::from(args);

    if let Some((old, new)) = &config.compare {
        match count_lines_cli::compare::compare_snapshots(old, new) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Comparison Error: {e}");
                ExitCode::FAILURE
            }
        }
    } else if config.watch {
        if let Err(e) = count_lines_cli::watch::watch_paths(&config) {
            eprintln!("Watch Error: {e}");
            ExitCode::FAILURE
        } else {
            // Watch loop is infinite, but if it returns Ok, it means it finished (unlikely)
            ExitCode::SUCCESS
        }
    } else {
        match engine::run(&config) {
            Ok(result) => {
                // Print any processing errors to stderr
                for (path, err) in &result.errors {
                    eprintln!("Error processing {}: {err}", path.display());
                }

                // Print successful results
                presentation::print_results(&result.stats, &config);

                // Return success even if some files had errors (non-strict mode)
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Application Error: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
