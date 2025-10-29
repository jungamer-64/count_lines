use std::{
    fmt,
    ops::{Add, AddAssign},
};

use serde::{Deserialize, Serialize};

/// 行数を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LineCount(usize);

impl LineCount {
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Default for LineCount {
    fn default() -> Self {
        Self::zero()
    }
}

impl Add for LineCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for LineCount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<usize> for LineCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for LineCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 文字数を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CharCount(usize);

impl CharCount {
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Default for CharCount {
    fn default() -> Self {
        Self::zero()
    }
}

impl Add for CharCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for CharCount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<usize> for CharCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for CharCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 単語数を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordCount(usize);

impl WordCount {
    #[inline]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl Default for WordCount {
    fn default() -> Self {
        Self::zero()
    }
}

impl Add for WordCount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for WordCount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<usize> for WordCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for WordCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_count_operations() {
        let a = LineCount::new(10);
        let b = LineCount::new(5);

        assert_eq!(a.value(), 10);
        assert_eq!((a + b).value(), 15);

        let mut c = a;
        c += b;
        assert_eq!(c.value(), 15);
    }

    #[test]
    fn char_count_zero() {
        let zero = CharCount::zero();
        assert!(zero.is_zero());
        assert_eq!(zero.value(), 0);
    }

    #[test]
    fn word_count_display() {
        let count = WordCount::new(42);
        assert_eq!(format!("{}", count), "42");
    }
}
