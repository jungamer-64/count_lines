// crates/shared-kernel/tests/filesize_huge.rs
use count_lines_shared_kernel::FileSize;

#[test]
fn to_human_tib_boundary() {
    let one_tib = 1024_u64.pow(4);
    assert_eq!(FileSize::from(one_tib - 1).to_human(), "1024.0 GiB");
    assert_eq!(FileSize::from(one_tib).to_human(), "1.0 TiB");
}
