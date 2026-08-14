//! Security test: plain-format output must not emit terminal-active or
//! record-forging bytes from attacker-influenced secret data.
//!
//! VULN-007 (CWE-117): a `\n` inside a secret value forges a whole new record
//! on the shared cross-secret `search` stream, attributed to a different
//! secret; a `\r` erases the preceding row in a terminal.

use goldfinch::cli::OutputFormat;
use goldfinch::commands::write_search;
use serde_json::json;
use std::collections::BTreeMap;

/// Bytes that must never reach a rendered plain-output line.
fn offending_bytes(s: &str) -> Vec<char> {
    s.chars()
        .filter(|c| {
            let u = *c as u32;
            u < 0x20                        // C0 controls, incl. \n \r \x1b
                || u == 0x7f                // DEL
                || (0x80..=0x9f).contains(&u) // C1 controls
                || (0x202a..=0x202e).contains(&u) // bidi embedding/override
                || (0x2066..=0x2069).contains(&u) // bidi isolates
                || u == 0x2028 || u == 0x2029 // line/paragraph separators
        })
        .collect()
}

fn render_search_plain(secret: &str, key: &str, value: &str) -> String {
    let mut inner = BTreeMap::new();
    inner.insert(key.to_string(), json!(value));
    let mut secrets = BTreeMap::new();
    secrets.insert(secret.to_string(), inner);

    let mut buf: Vec<u8> = Vec::new();
    write_search(&mut buf, &secrets, key, OutputFormat::Plain).expect("render must succeed");
    String::from_utf8(buf).expect("output must be valid UTF-8")
}

#[test]
fn planted_newline_cannot_forge_an_extra_record() {
    // The scan's payload: one logical record that forges a second line
    // attributed to a different secret, then erases the row before it.
    let out = render_search_plain(
        "attacker-owned",
        "token",
        "benign\nprod/db: FORGED-BY-ATTACKER\rERASED-PREV-ROW",
    );

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "one planted record must render as exactly one line, got {}: {:?}",
        lines.len(),
        lines
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("prod/db:")),
        "attacker forged a record attributed to another secret: {out:?}"
    );
}

#[test]
fn control_and_bidi_bytes_never_reach_plain_output() {
    let out = render_search_plain(
        "attacker-owned",
        "token",
        "a\nb\rc\x1b[31md\x7fe\u{202e}f\u{2028}g",
    );

    let found = offending_bytes(out.trim_end_matches('\n'));
    assert!(
        found.is_empty(),
        "plain output leaked terminal-active bytes {found:?} in {out:?}"
    );
}
