// crates/shared-kernel/tests/counts_sum.rs
use count_lines_shared_kernel::{CharCount, LineCount, WordCount};

#[test]
fn linecount_sum() {
    let total = [1usize, 2, 3].into_iter().map(LineCount::from).sum::<LineCount>();
    assert_eq!(usize::from(total), 6);
}

#[test]
fn charcount_sum_ref() {
    let values = [CharCount::from(5), CharCount::from(7)];
    let total: CharCount = values.iter().sum();
    assert_eq!(usize::from(total), 12);
}

#[test]
fn wordcount_add_assign() {
    let mut words = WordCount::from(10);
    words += WordCount::from(5);
    assert_eq!(usize::from(words), 15);
    words += 5usize;
    assert_eq!(words, 20usize);
}

#[test]
fn linecount_mixed_arithmetic() {
    let mut lines = LineCount::from(2);
    let next = lines + 3usize;
    assert_eq!(next, 5usize);
    lines += 4usize;
    assert_eq!(lines, LineCount::from(6));
}

#[test]
fn collect_from_usize_iterator() {
    let collected: LineCount = [1usize, 2, 3].into_iter().collect();
    assert_eq!(usize::from(collected), 6);

    let collected_counts: CharCount = [CharCount::from(1), CharCount::from(2)].into_iter().collect();
    assert_eq!(usize::from(collected_counts), 3);
}

#[test]
fn sum_usize_into_counts() {
    let lines: LineCount = [1usize, 2, 3].into_iter().sum();
    assert_eq!(usize::from(lines), 6);

    let chars: CharCount = [4usize, 5].into_iter().sum();
    assert_eq!(usize::from(chars), 9);
}
