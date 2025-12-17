// Crate-level lint configuration
//
// These lints are allowed at the crate level with justification:
// - multiple_crate_versions: Dependency version conflicts are out of our control
// - must_use_candidate: Many internal functions don't benefit from #[must_use]
// - missing_const_for_fn: Adding const requires careful consideration of stability
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]

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
