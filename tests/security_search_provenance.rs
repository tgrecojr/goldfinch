//! Security test: a search result must unambiguously identify which secret a
//! matched key came from.
//!
//! VULN-003 (CWE-1289): `format!("{}/{}", secret_name, key)` is not injective,
//! because `/` is legal in both AWS secret names and JSON keys. The pair
//! (secret "prod", key "db/password") and the pair (secret "prod/db", key
//! "password") both rendered as the identical string "prod/db/password", so an
//! attacker with write access to "prod" could forge attribution to the
//! unreadable secret "prod/db".

use goldfinch::cli::OutputFormat;
use goldfinch::commands::write_search;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn render_json(secret: &str, key: &str, value: &str, pattern: &str) -> Value {
    let mut inner = BTreeMap::new();
    inner.insert(key.to_string(), json!(value));
    let mut secrets = BTreeMap::new();
    secrets.insert(secret.to_string(), inner);

    let mut buf: Vec<u8> = Vec::new();
    write_search(&mut buf, &secrets, pattern, OutputFormat::Json).expect("render must succeed");
    serde_json::from_slice(&buf).expect("json arm must emit valid JSON")
}

#[test]
fn colliding_secret_and_key_pairs_are_distinguishable() {
    // The scan's exact scenario. Attacker writes key "db/password" into the
    // secret "prod"; the victim secret "prod/db" has a legitimate "password".
    //
    // Both are rendered with the SAME value, so the records can only differ by
    // how they identify provenance. Under the vulnerable join both collapse to
    // the identical identifier "prod/db/password".
    let attacker = render_json("prod", "db/password", "same-value", "password");
    let victim = render_json("prod/db", "password", "same-value", "password");

    assert_ne!(
        attacker, victim,
        "attacker and victim records are indistinguishable, so provenance can be forged: \
         {attacker:?}"
    );
}

#[test]
fn search_records_carry_secret_and_key_as_separate_fields() {
    let record = render_json("prod", "db/password", "attacker-supplied", "password");
    let first = &record.as_array().expect("records must be an array")[0];

    assert_eq!(
        first.get("secret").and_then(Value::as_str),
        Some("prod"),
        "record must name the owning secret in its own field: {first:?}"
    );
    assert_eq!(
        first.get("key").and_then(Value::as_str),
        Some("db/password"),
        "record must name the matched key in its own field: {first:?}"
    );
}

#[test]
fn secret_name_matches_are_still_distinguishable_from_key_matches() {
    // A match on the secret NAME is a different kind of record from a match on
    // a key within a secret; the two must not be confusable either.
    let mut inner = BTreeMap::new();
    inner.insert("unrelated".to_string(), json!("v"));
    let mut secrets = BTreeMap::new();
    secrets.insert("app-password-store".to_string(), inner);

    let mut buf: Vec<u8> = Vec::new();
    write_search(&mut buf, &secrets, "password", OutputFormat::Json).expect("render must succeed");
    let records: Value = serde_json::from_slice(&buf).expect("valid JSON");
    let first = &records.as_array().unwrap()[0];

    assert_eq!(
        first.get("secret").and_then(Value::as_str),
        Some("app-password-store"),
        "a secret-name match must still identify the secret in its own field: {first:?}"
    );
    assert!(
        first.get("key").map(Value::is_null).unwrap_or(true),
        "a secret-name match has no matched key, so `key` must be absent/null: {first:?}"
    );
}
