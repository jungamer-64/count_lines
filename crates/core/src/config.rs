use alloc::string::String;
use alloc::vec::Vec;
use hashbrown::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AnalysisConfig {
    pub count_words: bool,
    pub count_sloc: bool,
    pub count_newlines_in_chars: bool,
    pub map_ext: HashMap<String, String>,
}
