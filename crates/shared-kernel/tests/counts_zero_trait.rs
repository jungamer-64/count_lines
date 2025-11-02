// crates/shared-kernel/tests/counts_zero_trait.rs
use count_lines_shared_kernel::{CharCount, LineCount, WordCount};
use num_traits::Zero;

#[test]
fn zero_trait_consistency() {
    let mut line = LineCount::from(5);
    line.set_zero();
    assert!(line.is_zero());
    assert_eq!(line, LineCount::ZERO);
    assert_eq!(LineCount::zero(), LineCount::ZERO);

    assert_eq!(CharCount::zero(), CharCount::ZERO);
    assert!(CharCount::ZERO.is_zero());

    assert_eq!(WordCount::zero(), WordCount::ZERO);
    assert!(WordCount::ZERO.is_zero());
}

#[test]
fn default_matches_zero() {
    assert_eq!(LineCount::default(), LineCount::zero());
    assert!(LineCount::default().is_zero());

    assert_eq!(CharCount::default(), CharCount::zero());
    assert_eq!(WordCount::default(), WordCount::zero());
}
