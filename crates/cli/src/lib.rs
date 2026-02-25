// crates/cli/src/lib.rs
// 依存関係の推移的依存により複数のバージョンが混在するための抑制
// bitflags: same-file(1.x) vs crossterm/notify(2.x)
// windows-sys: notify/terminal_size(0.60) vs clap(0.61)
#![allow(clippy::multiple_crate_versions)]

pub mod args;
pub mod compare;
pub mod config;
pub mod error;
pub mod options;
pub mod parsers;
pub mod presentation;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
