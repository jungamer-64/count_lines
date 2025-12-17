# Project Roadmap

This roadmap outlines upcoming milestones for `count_lines`.

## Immediate

- Integrate incremental cache/watch with CI workflows (diff-based PR guards and fast checks).

## Short term

- Flesh out integration tests and formatter coverage.
- Further reduce `Config` boolean proliferation with enum-based options.

## Mid term

- Introduce persistence abstractions for alternate storage backends.
- **Engine responsibility separation**: Split `engine.rs` into `reader.rs` (file I/O, binary detection), `counter.rs` (statistics calculation), and `walker.rs` (file system coordination) for improved testability.

## Long term

- Explore feature-specific modules (e.g., Git integration) behind crate features.
- **Language definition externalization**: Migrate language definitions (extensions, comment styles) to a configuration file format (`languages.toml`) or use `phf` (Perfect Hash Function) maps for faster lookup and easier customization.
- **Word counting optimization**: Investigate SIMD-based approaches (similar to `bytecount`) for faster word counting on large files.
