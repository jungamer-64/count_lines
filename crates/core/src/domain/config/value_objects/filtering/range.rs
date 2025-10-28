
/// 範囲チェック用の値オブジェクト
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub min: Option<usize>,
    pub max: Option<usize>,
}

impl Range {
    /// 新しい範囲を作成
    #[must_use]
    pub const fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    /// 値が範囲内かチェック
    #[must_use]
    pub const fn contains(&self, value: usize) -> bool {
        let min_ok = match self.min {
            Some(min) => value >= min,
            None => true,
        };
        let max_ok = match self.max {
            Some(max) => value <= max,
            None => true,
        };
        min_ok && max_ok
    }

    /// 制約がないか確認
    #[must_use]
    pub const fn is_unconstrained(&self) -> bool {
        self.min.is_none() && self.max.is_none()
    }
}