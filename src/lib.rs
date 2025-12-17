// Crate-level lint configuration
// These are stylistic lints that are intentionally not addressed:
// - must_use_candidate: Internal helper functions don't need #[must_use]
// - missing_const_for_fn: Many simple functions could be const but aren't critical
// - missing_errors_doc: Internal functions with clear error semantics
// - missing_panics_doc: Internal functions with clear panic conditions
// - uninlined_format_args: Explicit format args are sometimes clearer
// - format_push_string: Current patterns are readable, can optimize later
// - items_after_statements: Some patterns are clearer with local functions
// - or_fun_call: Lazy evaluation not critical for performance here
// - useless_let_if_seq: Current patterns are clear
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::format_push_string)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::useless_let_if_seq)]
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
