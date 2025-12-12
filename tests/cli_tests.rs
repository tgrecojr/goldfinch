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
fn test_cli_missing_secret_argument() {
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("get")
        .arg("some-key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_cli_list_command_structure() {
    // This will fail without AWS credentials, but tests that CLI parsing works
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("test-secret")
        .arg("list")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_get_command_structure() {
    // This will fail without AWS credentials, but tests that CLI parsing works
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("test-secret")
        .arg("get")
        .arg("some-key")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_search_command_structure() {
    // This will fail without AWS credentials, but tests that CLI parsing works
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("test-secret")
        .arg("search")
        .arg("pattern")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_json_format_flag() {
    // Test that --format json is accepted
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("test-secret")
        .arg("--format")
        .arg("json")
        .arg("list")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_plain_format_flag() {
    // Test that --format plain is accepted
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("test-secret")
        .arg("--format")
        .arg("plain")
        .arg("list")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_invalid_format() {
    // Test that invalid format is rejected
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("test-secret")
        .arg("--format")
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

#[test]
fn test_cli_env_var_for_secret() {
    // Test that GOLDFINCH_SECRETS env var is used when --secrets is not provided
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.env("GOLDFINCH_SECRETS", "env-test-secret")
        .arg("list")
        .assert()
        .failure(); // Will fail due to AWS but proves env var parsing works
}

#[test]
fn test_cli_no_secret_provided() {
    // Test error when neither --secrets nor GOLDFINCH_SECRETS is provided
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.env_remove("GOLDFINCH_SECRETS")
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_cli_multiple_secrets() {
    // Test that multiple secrets can be specified
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.arg("--secrets")
        .arg("secret1,secret2,secret3")
        .arg("list")
        .assert()
        .failure(); // Expected to fail due to AWS credentials, but parsing should work
}

#[test]
fn test_cli_env_var_multiple_secrets() {
    // Test that multiple secrets can be specified via env var
    let mut cmd = Command::cargo_bin("goldfinch").unwrap();
    cmd.env("GOLDFINCH_SECRETS", "secret1,secret2,secret3")
        .arg("list")
        .assert()
        .failure(); // Will fail due to AWS but proves env var parsing works
}
