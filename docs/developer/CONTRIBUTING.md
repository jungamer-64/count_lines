# Contributing to `count_lines`

Thanks for your interest in improving `count_lines`! This project welcomes bug reports, feature proposals, and code contributions. The sections below describe how to get started.

## Prerequisites

- Rust toolchain (stable) with `cargo` available
- `cargo fmt`, `cargo clippy`, and `cargo test` installed via `rustup component add`
- Optional: GNU Make (for custom scripts) or your preferred editor/IDE

## Development Workflow

1. Fork the repository and create a feature branch.
2. Run `cargo fmt` before committing to keep formatting consistent.
3. Run `cargo check` frequently; submit PRs that pass `cargo test`.
4. Add or update tests when fixing bugs or adding functionality.
5. Document user-facing changes in `README.md`, `usage.txt`, or the changelog (if relevant).

### Useful Commands

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo check
cargo test -p count_lines_core
```

## Coding Guidelines

- Rust 2024 edition rules apply.
- Prefer clear, maintainable code with concise comments for complex logic.
- Use module naming conventions already established in the project (parent-file modules with subfolders).
- Keep CLI help output (see `usage.txt`) in sync when altering options or behaviour.
- Respect the clean architecture layering: keep core types in `shared/` and `domain/`, use cases in `application/`, adapters in `infrastructure/`, presentation logic in `presentation/`, and wire everything from `bootstrap/`.

## Submitting Changes

- Follow conventional commit messages when reasonable (e.g. `feat: add XYZ`, `fix: handle ABC`).
- Write descriptive PR titles and include reproduction steps or screenshots when relevant.
- Reference any related issues in the PR description.
- Ensure CI (format, lint, test) passes; note any intentionally skipped tests with justification.

## Reporting Issues

When filing issues, please provide:

- A clear description of the bug or feature request
- Steps to reproduce (for bugs)
- Environment details (`count_lines --version`, OS)
- Sample input/output if applicable

## License

By contributing, you agree that your contributions will be licensed under the dual MIT/Apache-2.0 terms used by the project. See `LICENSE-MIT` and `LICENSE-APACHE` for details.

Thanks again for helping make `count_lines` better!
