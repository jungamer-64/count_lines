/// Generic range value object for inclusive bounds checks.
#[derive(Debug, Default, Clone, Copy)]
pub struct Range {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl Range {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, v: usize) -> bool {
        self.min.is_none_or(|m| v >= m) && self.max.is_none_or(|x| v <= x)
    }
}
