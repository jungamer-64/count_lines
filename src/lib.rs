// Crate-level lint configuration
//
// These lints are allowed at the crate level with justification:
// - multiple_crate_versions: Dependency version conflicts are out of our control
// - must_use_candidate: Many internal functions don't benefit from #[must_use]
// - missing_const_for_fn: Adding const requires careful consideration of stability
// - missing_errors_doc: Internal helper functions with obvious error semantics
// - missing_panics_doc: Internal functions with clear panic conditions
// - items_after_statements: Local helper functions within larger parsing functions
// - or_fun_call: Lazy eval not critical for performance in this context
// - unused_self: Some methods keep &self for API consistency
// - unnecessary_wraps: Some return types are for future extensibility
// - format_push_string: Works correctly, low priority
// - too_many_lines: Complex functions are documented, will refactor incrementally
// - struct_excessive_bools: State tracking requires multiple flags
// - doc_markdown: Will address incrementally
// - field_reassign_with_default: Test setup patterns

#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::format_push_string)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::field_reassign_with_default)]

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
