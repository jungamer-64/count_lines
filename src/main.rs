#![allow(clippy::multiple_crate_versions)]

//! CLI entry point for the `count_lines` application.

fn main() -> anyhow::Result<()> {
    count_lines::run_from_cli()
}
