// crates/core/src/domain/config/value_objects/filtering/range.rs
/// Inclusive range helper used for count-based filtering.
#[derive(Debug, Default, Clone, Copy)]
pub struct Range {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl Range {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn contains(&self, value: usize) -> bool {
        self.min.is_none_or(|m| value >= m) && self.max.is_none_or(|m| value <= m)
    }
}
