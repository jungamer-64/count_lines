// crates/shared-kernel/tests/filesize_human.rs
use count_lines_shared_kernel::FileSize;

#[test]
fn human_boundaries() {
    assert_eq!(FileSize::from(1023).to_human(), "1023 B");
    assert_eq!(FileSize::from(1024).to_human(), "1.0 KiB");
    assert_eq!(FileSize::from(1536).to_human(), "1.5 KiB");
    assert_eq!(FileSize::from(1024 * 1024).to_human(), "1.0 MiB");
}
