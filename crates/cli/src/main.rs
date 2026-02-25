use clap::Parser;
use count_lines_cli::args::Args;
use count_lines_cli::config::Config;
use count_lines_cli::presentation;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();
    // Convert args to engine::Config
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
        // Define the callback for the watch loop
        let run_cycle = || {
            // Clear screen if WatchOutput is Full
            if matches!(
                config.watch_output,
                count_lines_engine::options::WatchOutput::Full
            ) {
                print!("\x1B[2J\x1B[1;1H");
            }
            
            match count_lines_engine::run(&config) {
                 Ok(result) => {
                    for (path, err) in &result.errors {
                        eprintln!("Error processing {}: {err}", path.display());
                    }
                    presentation::print_results(&result.stats, &config);
                 }
                 Err(e) => eprintln!("Error in watch cycle: {e}"),
            }
        };

        if let Err(e) = count_lines_engine::watch::watch_loop(&config, run_cycle) {
            eprintln!("Watch Error: {e}");
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        }
    } else {
        match count_lines_engine::run(&config) {
            Ok(result) => {
                for (path, err) in &result.errors {
                    eprintln!("Error processing {}: {err}", path.display());
                }

                presentation::print_results(&result.stats, &config);
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Application Error: {e}");
                ExitCode::FAILURE
            }
        }
    }
}
