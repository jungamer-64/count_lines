// crates/core/src/config.rs
use alloc::string::String;

use hashbrown::HashMap;

/// Configuration for content analysis.
#[derive(Debug, Clone, Default)]
pub struct AnalysisConfig {
    /// Whether to count words.
    pub count_words: bool,
    /// Whether to count SLOC (Source Lines of Code).
    pub count_sloc: bool,
    /// Whether to include newlines in character count.
    pub count_newlines_in_chars: bool,
    /// Extension mapping (e.g. `h` → `cpp`).
    pub map_ext: HashMap<String, String>,
}
