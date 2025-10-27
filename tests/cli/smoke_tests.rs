use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn shows_help() {
    Command::cargo_bin("count_lines")
        .expect("binary exists")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("count_lines"));
}

#[test]
fn processes_single_file() {
    Command::cargo_bin("count_lines")
        .expect("binary exists")
        .args(["--format", "json", "Cargo.toml"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"files\""));
}
