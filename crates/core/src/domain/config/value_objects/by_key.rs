use crate::domain::grouping::Granularity;

pub type GlobPattern = glob::Pattern;

/// Value object describing grouping keys for summarisation.
#[derive(Debug, Clone, Copy)]
pub enum ByKey {
    Ext,
    Dir(usize),
    Mtime(Granularity),
}
