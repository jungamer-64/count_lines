# Module Organization

This document describes how `count_lines` modules are organized.

## Module Structure

```text
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library exports
├── args.rs          # clap argument definitions
├── config.rs        # Runtime configuration
├── engine.rs        # Processing orchestration
├── filesystem.rs    # File discovery & traversal
├── stats.rs         # Statistics data structure
├── options.rs       # Enums (OutputFormat, OutputMode, SortKey, etc.)
├── parsers.rs       # Custom CLI parsers (size, date)
├── presentation.rs  # Output formatting (table, CSV, JSON, etc.)
├── compare.rs       # Snapshot comparison
├── watch.rs         # File watch mode
├── error.rs         # Error types
└── language/        # SLOC processing
    ├── mod.rs       # SlocProcessor dispatcher
    ├── processor_trait.rs
    ├── comment_style.rs
    ├── string_utils.rs
    └── processors/  # Per-language implementations
```

## Dependency Direction

```text
main.rs
   ↓
args.rs → config.rs → engine.rs → filesystem.rs
                         ↓
                    language/
                         ↓
                   presentation.rs
```

All modules depend inward toward the core types (`Config`, `FileStats`). External dependencies (`clap`, `rayon`, `ignore`) are isolated to their respective modules.

## See Also

- [ARCHITECTURE.md](../ARCHITECTURE.md) - Full architecture documentation
- [ROADMAP.md](../../project/ROADMAP.md) - Future structural improvements
