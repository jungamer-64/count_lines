/// Value object capturing size constraints for filtering.
#[derive(Debug, Default, Clone, Copy)]
pub struct SizeRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

impl SizeRange {
    pub fn new(min: Option<u64>, max: Option<u64>) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, v: u64) -> bool {
        self.min.is_none_or(|m| v >= m) && self.max.is_none_or(|x| v <= x)
    }
}
