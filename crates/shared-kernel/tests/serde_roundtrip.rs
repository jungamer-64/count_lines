// crates/shared-kernel/tests/serde_roundtrip.rs
use count_lines_shared_kernel::{FileSize, LineCount};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Wrapper {
    lines: LineCount,
    size: FileSize,
}

#[test]
fn json_roundtrip() {
    let original = Wrapper { lines: LineCount::from(42), size: FileSize::from(2048) };
    let json = serde_json::to_string(&original).expect("serializes");
    let decoded: Wrapper = serde_json::from_str(&json).expect("deserializes");
    assert_eq!(decoded, original);
}
