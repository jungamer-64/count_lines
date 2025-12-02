// crates/shared-kernel/src/value_objects/counts.rs
use std::{
    iter::{FromIterator, Sum},
    ops::{Add, AddAssign},
};

use num_traits::Zero;
use serde::{Deserialize, Serialize};

/// Represents a number of lines while keeping APIs type-safe and ergonomic.
///
/// - `+` / `+=` mirror plain `usize` arithmetic for speed.
/// - `saturating_*` helpers protect against overflow without leaving the type.
/// - Implements `Sum`, `FromIterator`, and `num_traits::Zero` so it blends into generic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[must_use]
#[repr(transparent)]
#[serde(transparent)]
pub struct LineCount(usize);

impl LineCount {
    pub const ZERO: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Zero for LineCount {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl Add for LineCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for LineCount {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<&LineCount> for LineCount {
    #[inline]
    fn add_assign(&mut self, rhs: &LineCount) {
        self.0 += rhs.0;
    }
}
impl PartialEq<usize> for LineCount {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}
impl PartialEq<LineCount> for usize {
    fn eq(&self, other: &LineCount) -> bool {
        *self == other.0
    }
}
impl Add<usize> for LineCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl AddAssign<usize> for LineCount {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl From<usize> for LineCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl From<LineCount> for usize {
    fn from(value: LineCount) -> Self {
        value.value()
    }
}

impl From<std::num::NonZeroUsize> for LineCount {
    fn from(value: std::num::NonZeroUsize) -> Self {
        Self::new(value.get())
    }
}
impl FromIterator<LineCount> for LineCount {
    fn from_iter<I: IntoIterator<Item = LineCount>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}
impl FromIterator<usize> for LineCount {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}
impl Sum for LineCount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}
impl<'a> Sum<&'a LineCount> for LineCount {
    fn sum<I: Iterator<Item = &'a LineCount>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + *item)
    }
}
impl Sum<usize> for LineCount {
    fn sum<I: Iterator<Item = usize>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}

/// Tracks character counts with the same semantics as `LineCount`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[must_use]
#[repr(transparent)]
#[serde(transparent)]
pub struct CharCount(usize);

impl CharCount {
    pub const ZERO: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Zero for CharCount {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl Add for CharCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for CharCount {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl PartialEq<usize> for CharCount {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}
impl PartialEq<CharCount> for usize {
    fn eq(&self, other: &CharCount) -> bool {
        *self == other.0
    }
}
impl Add<usize> for CharCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl AddAssign<usize> for CharCount {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl From<usize> for CharCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl From<CharCount> for usize {
    fn from(value: CharCount) -> Self {
        value.value()
    }
}
impl FromIterator<CharCount> for CharCount {
    fn from_iter<I: IntoIterator<Item = CharCount>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}
impl FromIterator<usize> for CharCount {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}
impl Sum for CharCount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}
impl<'a> Sum<&'a CharCount> for CharCount {
    fn sum<I: Iterator<Item = &'a CharCount>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + *item)
    }
}
impl Sum<usize> for CharCount {
    fn sum<I: Iterator<Item = usize>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}

/// Represents word totals while preserving the ergonomic arithmetic helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[must_use]
#[repr(transparent)]
#[serde(transparent)]
pub struct WordCount(usize);

impl WordCount {
    pub const ZERO: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Zero for WordCount {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl Add for WordCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for WordCount {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl PartialEq<usize> for WordCount {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}
impl PartialEq<WordCount> for usize {
    fn eq(&self, other: &WordCount) -> bool {
        *self == other.0
    }
}
impl Add<usize> for WordCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl AddAssign<usize> for WordCount {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl From<usize> for WordCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}
impl From<WordCount> for usize {
    fn from(value: WordCount) -> Self {
        value.value()
    }
}
impl FromIterator<WordCount> for WordCount {
    fn from_iter<I: IntoIterator<Item = WordCount>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}
impl FromIterator<usize> for WordCount {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}
impl Sum for WordCount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}
impl<'a> Sum<&'a WordCount> for WordCount {
    fn sum<I: Iterator<Item = &'a WordCount>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + *item)
    }
}
impl Sum<usize> for WordCount {
    fn sum<I: Iterator<Item = usize>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}

/// SLOC (Source Lines of Code) - 空行を除外した純粋コード行数を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[must_use]
#[repr(transparent)]
#[serde(transparent)]
pub struct SlocCount(usize);

impl SlocCount {
    pub const ZERO: Self = Self(0);

    #[inline]
    #[must_use]
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(0)
    }

    #[inline]
    #[must_use]
    pub const fn value(self) -> usize {
        self.0
    }

    #[inline]
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add(self, rhs: usize) -> Self {
        Self(self.0.saturating_add(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub(self, rhs: usize) -> Self {
        Self(self.0.saturating_sub(rhs))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_add_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    #[inline]
    #[must_use]
    pub const fn saturating_sub_count(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Zero for SlocCount {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self.0 == 0
    }

    fn set_zero(&mut self) {
        self.0 = 0;
    }
}

impl Add for SlocCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for SlocCount {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl PartialEq<usize> for SlocCount {
    fn eq(&self, other: &usize) -> bool {
        self.0 == *other
    }
}

impl PartialEq<SlocCount> for usize {
    fn eq(&self, other: &SlocCount) -> bool {
        *self == other.0
    }
}

impl Add<usize> for SlocCount {
    type Output = Self;

    #[inline]
    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for SlocCount {
    #[inline]
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}

impl From<usize> for SlocCount {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<SlocCount> for usize {
    fn from(value: SlocCount) -> Self {
        value.value()
    }
}

impl FromIterator<SlocCount> for SlocCount {
    fn from_iter<I: IntoIterator<Item = SlocCount>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}

impl FromIterator<usize> for SlocCount {
    fn from_iter<I: IntoIterator<Item = usize>>(iter: I) -> Self {
        iter.into_iter().fold(Self::zero(), |acc, n| acc + n)
    }
}

impl Sum for SlocCount {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}

impl<'a> Sum<&'a SlocCount> for SlocCount {
    fn sum<I: Iterator<Item = &'a SlocCount>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + *item)
    }
}

impl Sum<usize> for SlocCount {
    fn sum<I: Iterator<Item = usize>>(iter: I) -> Self {
        iter.fold(Self::zero(), |acc, item| acc + item)
    }
}

mod display {
    use std::fmt;

    use super::{CharCount, LineCount, SlocCount, WordCount};

    impl fmt::Display for LineCount {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    impl fmt::Display for CharCount {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    impl fmt::Display for WordCount {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value())
        }
    }

    impl fmt::Display for SlocCount {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value())
        }
    }
}
