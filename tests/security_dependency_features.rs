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
    // `sso` is deliberately re-enabled: AWS SSO profiles are a supported auth
    // path for goldfinch and the SSO provider carries no shell-exec primitive.
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

/// VULN-006 (CWE-1395): the superseded `rustls-webpki 0.101.7` (three RUSTSEC
/// advisories) was resolved alongside the patched 0.103 line.
///
/// It arrives via `rustls 0.21` / `hyper-rustls 0.24`, which
/// `aws-smithy-http-client` links behind its `legacy-rustls-ring` feature.
/// That feature is turned on by `aws-sdk-secretsmanager`'s default `rustls`
/// feature -- not by an out-of-date version pin -- so the fix is to drop the
/// legacy feature, keeping the modern `default-https-client` (rustls 0.23 +
/// aws-lc) stack for TLS.
#[test]
fn advisory_bearing_tls_stack_is_not_resolved() {
    let lock = cargo_lock();

    assert!(
        !lock.contains("name = \"rustls-webpki\"\nversion = \"0.101."),
        "rustls-webpki 0.101.x is resolved; it carries three RUSTSEC advisories \
         and is superseded by the 0.103 line"
    );
    assert!(
        !lock.contains("name = \"rustls\"\nversion = \"0.21."),
        "rustls 0.21.x is resolved; it is the sole path to rustls-webpki 0.101.x"
    );
    assert!(
        !lock.contains("name = \"hyper-rustls\"\nversion = \"0.24."),
        "hyper-rustls 0.24.x is resolved; it pulls the legacy rustls 0.21 stack"
    );
}

/// Checks the *resolved* feature set of aws-config, not just the manifest line.
/// `credentials-process` adds no crate of its own, so Cargo.lock cannot act as
/// a proxy for it; `cargo metadata` reports the features Cargo actually turned
/// on after unifying every dependent's requests.
#[test]
fn credentials_process_feature_is_not_resolved() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = std::process::Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--offline"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata must run");
    assert!(out.status.success(), "cargo metadata failed");

    let meta: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("cargo metadata must emit JSON");
    let nodes = meta["resolve"]["nodes"]
        .as_array()
        .expect("resolve.nodes must be an array");

    let aws_config_nodes: Vec<&serde_json::Value> = nodes
        .iter()
        .filter(|n| n["id"].as_str().is_some_and(|id| id.contains("aws-config")))
        .collect();
    assert!(!aws_config_nodes.is_empty(), "aws-config must be resolved");

    for node in aws_config_nodes {
        let features: Vec<&str> = node["features"]
            .as_array()
            .map(|f| f.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();
        assert!(
            !features.contains(&"credentials-process"),
            "aws-config resolved with credentials-process enabled \
             (the `sh -c` credential provider); features: {features:?}"
        );
    }
}
