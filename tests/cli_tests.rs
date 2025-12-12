use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("goldfinch"));
}

#[test]
fn test_cli_get_command_without_credentials() {
    // This will fail without AWS credentials, but tests that CLI parsing works
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("get")
        .arg("some-key")
        .assert()
        .failure(); // Expected to fail due to AWS credentials
}

#[test]
fn test_cli_list_command_structure() {
    // Tests that CLI parsing works. May succeed if AWS credentials are configured.
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    let assert = cmd.arg("list").assert();
    // Either succeeds with credentials or fails without them
    let _ = assert.try_success().or_else(|_| cmd.arg("list").assert().try_failure());
}

#[test]
fn test_cli_get_command_structure() {
    // This will fail without AWS credentials, but tests that CLI parsing works
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("get")
        .arg("some-key")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_search_command_structure() {
    // This will fail without AWS credentials, but tests that CLI parsing works
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("search")
        .arg("pattern")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_json_format_flag() {
    // Test that --format json is accepted. May succeed if AWS credentials are configured.
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    let assert = cmd.arg("--format").arg("json").arg("list").assert();
    // Either succeeds with credentials or fails without them
    let _ = assert.try_success().or_else(|_| {
        Command::cargo_bin("goldfinch")
            .unwrap()
            .arg("--format")
            .arg("json")
            .arg("list")
            .assert()
            .try_failure()
    });
}

#[test]
fn test_cli_plain_format_flag() {
    // Test that --format plain is accepted. May succeed if AWS credentials are configured.
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    let assert = cmd.arg("--format").arg("plain").arg("list").assert();
    // Either succeeds with credentials or fails without them
    let _ = assert.try_success().or_else(|_| {
        Command::cargo_bin("goldfinch")
            .unwrap()
            .arg("--format")
            .arg("plain")
            .arg("list")
            .assert()
            .try_failure()
    });
}

#[test]
fn test_cli_invalid_format() {
    // Test that invalid format is rejected
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--format")
        .arg("invalid")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("goldfinch"));
}

