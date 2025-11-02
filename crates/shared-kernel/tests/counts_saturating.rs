// crates/shared-kernel/tests/counts_saturating.rs
use count_lines_shared_kernel::{CharCount, LineCount, WordCount};

#[test]
fn linecount_saturating_add_and_sub() {
    let max = LineCount::from(usize::MAX);
    assert_eq!(max.saturating_add(1), max);
    assert_eq!(LineCount::from(5).saturating_add_count(LineCount::from(usize::MAX)), max);
    let zeroed = LineCount::zero().saturating_sub(5);
    assert_eq!(zeroed, LineCount::zero());
    assert_eq!(LineCount::from(3).saturating_sub_count(LineCount::from(5)), LineCount::zero());
}

#[test]
fn charcount_saturating_add_and_sub() {
    let max = CharCount::from(usize::MAX);
    assert_eq!(max.saturating_add(42), max);
    assert_eq!(CharCount::from(3).saturating_sub(5), CharCount::zero());
    assert_eq!(CharCount::from(2).saturating_add_count(max), max);
    assert_eq!(CharCount::from(2).saturating_sub_count(CharCount::from(3)), CharCount::ZERO);
}

#[test]
fn wordcount_saturating_add_and_sub() {
    let max = WordCount::from(usize::MAX);
    assert_eq!(max.saturating_add(usize::MAX), max);
    assert_eq!(WordCount::from(2).saturating_sub(3), WordCount::zero());
    assert_eq!(WordCount::from(usize::MAX - 1).saturating_add_count(WordCount::from(5)), max);
    assert_eq!(WordCount::from(2).saturating_sub_count(WordCount::from(5)), WordCount::ZERO);
}
