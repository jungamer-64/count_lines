// crates/shared-kernel/tests/filesize_display.rs
use count_lines_shared_kernel::FileSize;

#[test]
fn display_alternate_is_human() {
    let value = FileSize::from(1536);
    assert_eq!(format!("{}", value), "1536");
    assert_eq!(format!("{:#}", value), "1.5 KiB");
}
