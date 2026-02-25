use assert_cmd::Command;
use insta::{assert_snapshot, assert_json_snapshot};
use serde_json::Value;

#[test]
fn test_help() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("count_lines").unwrap();
    cmd.arg("--help");
    let assert = cmd.assert();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_snapshot!(stdout);
}

#[test]
fn test_scan_sample_json() {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("count_lines").unwrap();
    cmd.current_dir(env!("CARGO_MANIFEST_DIR")); 
    cmd.arg("tests/fixtures/sample.rs").arg("--format").arg("json");

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let json: Value = serde_json::from_slice(&output.stdout).expect("Failed to parse JSON output");

    assert_json_snapshot!(json, {
        "[].mtime" => "[MTIME]",
    });
}
