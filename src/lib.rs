// 依存関係の推移的依存により複数のバージョンが混在するための抑制
// bitflags: same-file(1.x) vs crossterm/notify(2.x)
// windows-sys: notify/terminal_size(0.60) vs clap(0.61)
#![allow(clippy::multiple_crate_versions)]

pub mod args;
pub mod compare;
pub mod config;
pub mod engine;
pub mod error;
pub mod filesystem;
pub mod language;
pub mod options;
pub mod parsers;
pub mod presentation;
pub mod stats;
pub mod watch;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
