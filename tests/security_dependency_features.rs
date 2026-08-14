//! Security test: the build must not link aws-config's `credentials-process`
//! provider, which executes `credential_process` config values via `sh -c`.
//!
//! goldfinch never uses that provider, so the shell-exec primitive is unused
//! capability compiled into the binary.

use std::fs;

fn cargo_toml() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("Cargo.toml must be readable")
}

fn cargo_lock() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
        .expect("Cargo.lock must be readable")
}

/// Returns the single-line dependency declaration for `name`.
fn dep_line(manifest: &str, name: &str) -> String {
    manifest
        .lines()
        .find(|l| l.trim_start().starts_with(&format!("{name} ")))
        .unwrap_or_else(|| panic!("no `{name}` dependency line in Cargo.toml"))
        .to_string()
}

#[test]
fn aws_config_does_not_link_credentials_process_provider() {
    let line = dep_line(&cargo_toml(), "aws-config");

    assert!(
        line.contains("default-features = false"),
        "aws-config must set default-features = false so the credentials-process \
         provider (which runs `sh -c` on config values) is not linked; got: {line}"
    );
    assert!(
        !line.contains("credentials-process"),
        "aws-config must not re-enable credentials-process; got: {line}"
    );
    assert!(
        !line.contains("\"sso\""),
        "aws-config must not re-enable sso; got: {line}"
    );
}

/// VULN-008 (CWE-1164): `tokio = ["full"]` links process, signal, fs and net
/// capability that goldfinch never uses. `full` is also the surviving enabler
/// of `tokio/process` once VULN-002 removes aws-config's credentials-process.
///
/// Note: this asserts only what removing `full` actually controls. `tokio/fs`
/// and `tokio/net` stay enabled through aws-smithy-types' `rt-tokio` and
/// hyper respectively, so asserting their absence would be false.
#[test]
fn tokio_does_not_link_full_capability_set() {
    let line = dep_line(&cargo_toml(), "tokio");

    assert!(
        line.contains("default-features = false"),
        "tokio must set default-features = false; got: {line}"
    );
    assert!(
        !line.contains("\"full\""),
        "tokio must not enable the `full` meta-feature, which links unused \
         process/signal/fs/net capability; got: {line}"
    );
}

#[test]
fn sso_crates_are_not_resolved() {
    // aws-sdk-sso is reachable only via aws-config's default feature set.
    // Its presence in the lock proves the default set is active.
    assert!(
        !cargo_lock().contains("name = \"aws-sdk-sso\""),
        "aws-sdk-sso is resolved, which means aws-config's default feature set \
         (including credentials-process) is still linked"
    );
}
