use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn shows_help() {
    Command::new(env!("CARGO_BIN_EXE_count_lines"))
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("count_lines"));
}

#[test]
fn processes_single_file() {
    Command::new(env!("CARGO_BIN_EXE_count_lines"))
        .args(["--format", "json", "Cargo.toml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\""));
}
