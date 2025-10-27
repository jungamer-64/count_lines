# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Reorganized project structure for better maintainability
- Moved documentation to `docs/` directory
- Restructured tests into `cli/`, `integration/`, and `fixtures/` subdirectories
- Updated `.gitignore` to exclude log files and build artifacts

### Added
- `ARCHITECTURE.md` documenting the project's design and structure
- Helper scripts: `test.sh`, `benchmark.sh`, and `release.sh`
- Test fixtures directory with README
- Comprehensive `.gitignore` rules

## [0.5.0] - 2024-10-27

### Added
- Layered architecture (foundation → domain → interface → app)
- Workspace configuration with separate core library (`count_lines_core`)
- JSON/YAML/JSONL output with version field for snapshot comparison
- Snapshot comparison feature (`--compare old.json new.json`)
- Grouping by extension, directory depth, and mtime (`--by`)
- Expression-based filtering (`--filter "lines > 100 && ext == 'rs'"`)
- Progress indicator support (`--progress`)
- Ratio columns for percentage display (`--ratio`)
- Git mode with `.gitignore` support (`--git`)
- Multiple output formats: Table, CSV, TSV, JSON, YAML, Markdown, JSONL
- Comprehensive filtering options (size, lines, chars, words, mtime, glob)
- Multi-field sorting with ascending/descending order
- Word count support (`--words`)
- Hidden files option (`--hidden`)
- File list input support (`--files_from`)

### Changed
- Refactored to clean architecture with clear layer separation
- Improved parallel processing with Rayon
- Enhanced error handling with anyhow
- Better CLI argument parsing with clap v4.5

### Performance
- Optimized release builds with LTO and single codegen unit
- Parallel file processing for large repositories
- Efficient text file detection

## [0.4.0] - (Historical)

### Added
- Basic file counting functionality
- Multiple output format support
- Sorting capabilities

## [0.3.0] - (Historical)

### Added
- Recursive directory scanning
- Extension-based filtering
- CSV output support

## [0.2.0] - (Historical)

### Added
- Character counting
- Basic filtering options

## [0.1.0] - (Historical)

### Added
- Initial release
- Basic line counting for files
- Simple CLI interface

---

## Categories

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security improvements
- **Performance**: Performance improvements

[Unreleased]: https://github.com/jungamer-64/count_lines/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/jungamer-64/count_lines/releases/tag/v0.5.0