//! Security test: the `get --format plain` arm must not emit terminal-active
//! or record-forging bytes from attacker-influenced secret data.
//!
//! VULN-009 (CWE-150) is the same class as VULN-007 but a distinct sink: the
//! `get` render path. It has its own test so the two sinks cannot drift apart.

use goldfinch::cli::OutputFormat;
use goldfinch::commands::write_secret;
use serde_json::json;
use std::collections::BTreeMap;

fn render_get_plain(pairs: &[(&str, &str)]) -> String {
    let mut secret = BTreeMap::new();
    for (k, v) in pairs {
        secret.insert(k.to_string(), json!(v));
    }

    let mut buf: Vec<u8> = Vec::new();
    write_secret(&mut buf, &secret, OutputFormat::Plain).expect("render must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

#[test]
fn get_plain_emits_one_line_per_pair() {
    // A single pair whose value carries a newline must not become two records.
    let out = render_get_plain(&[("api_key", "real\nadmin_password: LEAKED")]);

    assert_eq!(
        out.lines().count(),
        1,
        "one key/value pair must render as exactly one line, got {:?}",
        out.lines().collect::<Vec<_>>()
    );
    assert!(
        !out.lines().any(|l| l.starts_with("admin_password:")),
        "attacker forged a second key/value record: {out:?}"
    );
}

#[test]
fn get_plain_escapes_terminal_active_bytes() {
    let out = render_get_plain(&[("api_key", "a\x1b[31mb\x7fc\u{202e}d")]);

    for (label, needle) in [
        ("ESC", '\u{1b}'),
        ("DEL", '\u{7f}'),
        ("bidi override", '\u{202e}'),
    ] {
        assert!(
            !out.contains(needle),
            "{label} reached plain output unescaped: {out:?}"
        );
    }
}

#[test]
fn get_plain_escapes_control_chars_in_keys_too() {
    // The key side is attacker-influenced as well: keys come from the secret's
    // own JSON, which whoever writes the secret controls.
    let out = render_get_plain(&[("evil\nforged", "value")]);

    assert_eq!(
        out.lines().count(),
        1,
        "a control character in a KEY must not forge a record: {:?}",
        out.lines().collect::<Vec<_>>()
    );
}
