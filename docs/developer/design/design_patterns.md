# Design Patterns

> **Note**: This document describes aspirational patterns. The current implementation uses a simpler, more direct approach. See [ROADMAP.md](../../project/ROADMAP.md) for planned improvements.

## Current Architecture

The current implementation uses a straightforward modular design:

1. **CLI Layer** (`args.rs`) - Argument parsing with clap
2. **Configuration** (`config.rs`) - Args → Config transformation
3. **Engine** (`engine.rs`) - Orchestrates file processing
4. **I/O** (`filesystem.rs`) - File system traversal
5. **Output** (`presentation.rs`) - Result formatting

## Future Considerations

### Responsibility Separation

The engine currently combines file reading, content processing, and parallel orchestration. Future refactoring could split into:

- `reader.rs` - File I/O, binary detection
- `counter.rs` - Statistics calculation (pure functions)
- `walker.rs` - File system coordination

### Configuration Builder

A Builder pattern could improve testability:

```rust
// Aspirational API
let config = Config::builder()
    .roots(vec!["./src"])
    .output_mode(OutputMode::Summary)
    .count_sloc(true)
    .build()?;
```

See [ROADMAP.md](../../project/ROADMAP.md) for the full improvement plan.
