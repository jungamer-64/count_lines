// crates/shared-kernel/tests/error_context.rs
use std::io;

use count_lines_shared_kernel::{CountLinesError, ErrorContext};

fn boom() -> std::result::Result<(), io::Error> {
    Err(io::Error::other("root-io"))
}

#[test]
fn context_wraps_and_formats() {
    let err = boom()
        .map_err(CountLinesError::from)
        .context("reading config")
        .unwrap_err();

    let display = err.to_string();
    assert!(display.contains("reading config"));
    assert!(display.contains("Output error:"));
}
