// crates/shared-kernel/tests/counts_eq.rs
use count_lines_shared_kernel::LineCount;

#[test]
fn eq_with_usize_both_sides() {
    let count = LineCount::from(7);
    assert!(count == 7usize);
    assert!(7usize == count);
}
