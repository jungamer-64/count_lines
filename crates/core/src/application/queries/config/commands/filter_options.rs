/// Command DTO capturing filter-specific options.
#[derive(Debug, Default)]
pub struct FilterOptions {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub include_path: Vec<String>,
    pub exclude_path: Vec<String>,
    pub exclude_dir: Vec<String>,
    pub ext: Option<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub min_lines: Option<usize>,
    pub max_lines: Option<usize>,
    pub min_chars: Option<usize>,
    pub max_chars: Option<usize>,
    pub min_words: Option<usize>,
    pub max_words: Option<usize>,
    pub filter: Option<String>,
}
