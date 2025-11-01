use count_lines_domain::model::FileEntry;

#[derive(Debug, Clone)]
pub struct CountEntriesOutput {
    pub files: Vec<FileEntry>,
}
