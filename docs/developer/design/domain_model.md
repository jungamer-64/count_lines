# Domain Model Overview

This document describes the key data structures in `count_lines`.

## Core Structures

### FileStats (`src/stats.rs`)

Holds calculated statistics for a single file:

```rust
pub struct FileStats {
    pub path: PathBuf,              // File path
    pub name: String,               // File name
    pub lines: usize,               // Total line count
    pub chars: usize,               // Character count
    pub words: Option<usize>,       // Word count (if enabled)
    pub sloc: Option<usize>,        // Source lines of code (if enabled)
    pub size: u64,                  // File size in bytes
    pub mtime: Option<DateTime>,    // Modification time
    pub is_binary: bool,            // Binary file flag
}
```

### Config (`src/config.rs`)

Runtime configuration combining walk options, filters, and output settings:

- `WalkOptions` - File system traversal settings
- `FilterConfig` - Include/exclude patterns, size/line limits
- `OutputMode` - Full, Summary, or TotalOnly
- Various counting and formatting options

### SlocProcessor (`src/language/mod.rs`)

Enum dispatch processor for language-specific SLOC counting. Each variant wraps a processor implementation:

- `CStyleProcessor` - C, C++, Java, Go
- `PythonProcessor` - Python (docstrings)
- `JavaScriptProcessor` - JS/TS (template literals)
- etc.

## Data Flow

```text
Config → engine::run() → Vec<FileStats> → presentation::print_results()
```

See [ARCHITECTURE.md](../ARCHITECTURE.md) for detailed data flow diagrams.
