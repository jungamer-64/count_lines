// Crate-level lint configuration
// 不要な allow を削除し、clippy の推奨に従う形にします。
// どうしても必要な箇所（依存関係の問題など）のみ残します。
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
