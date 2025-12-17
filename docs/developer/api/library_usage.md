# Library Usage

`count_lines` exposes its functionality as a library crate, allowing programmatic access to file counting features.

## Public API

The library exposes the following modules:

```rust
use count_lines::{
    args::Args,           // CLI argument definitions
    config::Config,       // Runtime configuration
    engine,               // File processing engine
    stats::FileStats,     // Statistics structure
    presentation,         // Output formatting
    options::OutputMode,  // Output mode enum
};
```

## Basic Usage

```rust
use count_lines::config::Config;
use count_lines::engine;
use count_lines::presentation;

fn main() {
    // Create a default configuration
    let mut config = Config::default();
    config.walk.roots = vec![std::path::PathBuf::from("./src")];

    // Run the analysis
    match engine::run(&config) {
        Ok(stats) => {
            // Print results using the built-in presentation
            presentation::print_results(&stats, &config);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Key Types

### `Config`

Central configuration structure. See [config.rs](file:///d:/Rust/count_lines/src/config.rs) for all options.

### `FileStats`

Holds per-file statistics including lines, characters, words, and SLOC.

### `SlocProcessor`

Language-aware source lines of code processor. Supports 20+ languages.

## See Also

- [src/lib.rs](file:///d:/Rust/count_lines/src/lib.rs) - Public module exports
- [src/config.rs](file:///d:/Rust/count_lines/src/config.rs) - Configuration structure
- [src/engine.rs](file:///d:/Rust/count_lines/src/engine.rs) - Main processing engine
