use clap::Parser;
use count_lines::args::Args;
use count_lines::config::Config;
use count_lines::engine;
use count_lines::presentation;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    let config = Config::from_args(args);

    if let Some((old, new)) = &config.compare {
        match count_lines::compare::compare_snapshots(old, new) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Comparison Error: {e}");
                ExitCode::FAILURE
            }
        }
    } else if config.watch {
        if let Err(e) = count_lines::watch::watch_paths(&config) {
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
