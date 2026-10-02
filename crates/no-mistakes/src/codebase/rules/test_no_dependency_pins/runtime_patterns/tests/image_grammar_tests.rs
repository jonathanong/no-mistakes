//! Docker references, registry hosts, and CI labels are ASCII-only. A non-ASCII
//! character ends a reference (it is a boundary, never part of a tag) and is
//! never a digit; a registry host label is RFC 1123.

use super::super::RUNTIME_PATTERNS;
use super::{assert_pins, pins};
use regex::Regex;

const DIGEST: &str = "e0ef56d6eb603d9ffe87e74c28b8210b0ea93839e2ea4fb64f1fc2c66e95e391";

/// Pins from the context-free patterns, before the synthetic-value exemption,
/// so a reserved host such as `localhost:5000` is still visible to the test.
fn raw_pins(line: &str) -> Vec<String> {
    RUNTIME_PATTERNS
        .iter()
        .filter(|(_, _, context)| context.is_none())
        .filter_map(|(_, regex, _)| Regex::new(regex).unwrap().captures(line))
        .map(|captures| captures["pin"].to_string())
        .collect()
}

/// Fails with every offending line, not just the first.
fn assert_none(lines: &[String]) {
    let reported: Vec<_> = lines
        .iter()
        .filter_map(|line| Some((line, pins(line))).filter(|(_, found)| !found.is_empty()))
        .collect();
    assert!(reported.is_empty(), "{reported:#?}");
}

fn owned(lines: &[&str]) -> Vec<String> {
    lines.iter().map(|line| (*line).to_string()).collect()
}

#[test]
fn a_non_ascii_character_after_a_reference_ends_it() {
    for (line, expected) in [
        ("ghcr.io/acme/api:v2β", "ghcr.io/acme/api:v2"),
        ("ghcr.io/acme/api:2β", "ghcr.io/acme/api:2"),
        ("ghcr.io/acme/api:2.1β", "ghcr.io/acme/api:2.1"),
        ("ghcr.io/acme/api:v2.1β", "ghcr.io/acme/api:v2.1"),
        ("ghcr.io/acme/api:2.1é", "ghcr.io/acme/api:2.1"),
        ("ghcr.io/acme/api:2.1-rc1β", "ghcr.io/acme/api:2.1-rc1"),
        ("ghcr.io/acme/api:2.1.β", "ghcr.io/acme/api:2.1"),
        ("image: postgres:18β", "postgres:18"),
        ("FROM postgres:18β", "postgres:18"),
        ("image: valkey/valkey-bundle:9β", "valkey/valkey-bundle:9"),
        ("image:ghcr.io/acme/api:v2β", "ghcr.io/acme/api:v2"),
        ("ubuntu-22.04β", "ubuntu-22.04"),
        ("macos-15β", "macos-15"),
    ] {
        assert_pins(line, &[expected]);
    }
    let digest = format!("ghcr.io/acme/api@sha256:{DIGEST}");
    assert_pins(&format!("{digest}β"), &[&digest]);
    let tagged = format!("ghcr.io/acme/api:2.1@sha256:{DIGEST}");
    assert_pins(&format!("{tagged}β"), &[&tagged]);
}

#[test]
fn a_non_ascii_character_before_a_reference_starts_it() {
    for (line, expected) in [
        ("βghcr.io/acme/api:2", "ghcr.io/acme/api:2"),
        ("βghcr.io/acme/api:v2", "ghcr.io/acme/api:v2"),
        ("éghcr.io/acme/api:2.1", "ghcr.io/acme/api:2.1"),
        ("βubuntu-22.04", "ubuntu-22.04"),
    ] {
        assert_pins(line, &[expected]);
    }
    let digest = format!("ghcr.io/acme/api@sha256:{DIGEST}");
    assert_pins(&format!("β{digest}"), &[&digest]);
}

#[test]
fn typographic_punctuation_and_text_around_a_reference_delimit_it() {
    for (line, expected) in [
        ("“ghcr.io/acme/api:2.1”", "ghcr.io/acme/api:2.1"),
        ("‘ghcr.io/acme/api:v2’", "ghcr.io/acme/api:v2"),
        ("「ghcr.io/acme/api:2.1」", "ghcr.io/acme/api:2.1"),
        ("“image: postgres:18", "postgres:18"),
        ("日本 ghcr.io/acme/api:2.1 語", "ghcr.io/acme/api:2.1"),
        ("café: ghcr.io/acme/api:2.1", "ghcr.io/acme/api:2.1"),
        ("image: postgres:18 # café", "postgres:18"),
        ("(ghcr.io/acme/api:2.1)", "ghcr.io/acme/api:2.1"),
    ] {
        assert_pins(line, &[expected]);
    }
}

#[test]
fn a_non_ascii_neighbor_affects_only_its_own_reference_boundary() {
    assert_pins(
        "ghcr.io/acme/a:1.2β ghcr.io/acme/b:3.4",
        &["ghcr.io/acme/a:1.2", "ghcr.io/acme/b:3.4"],
    );
    assert_pins(
        "ghcr.io/acme/a:1.2,βghcr.io/acme/b:3.4",
        &["ghcr.io/acme/a:1.2", "ghcr.io/acme/b:3.4"],
    );
}

#[test]
fn only_ascii_digits_are_version_digits() {
    let lines = owned(&[
        "ghcr.io/acme/api:٢.١",
        "ghcr.io/acme/api:٢",
        "ghcr.io/acme/api:v٢.١",
        "ghcr.io/acme/api:٢-alpine",
        "ghcr.io/acme/api:２",
        "ghcr.io/acme/api:pg١٨",
        "ghcr.io:٥٠٠٠/acme/api:2",
        "image: postgres:١٨",
        "image: valkey/valkey-bundle:٩",
        "FROM node:٢٤",
        "ubuntu-٢٢.٠٤",
        "ubuntu-٢٢.04",
        "ubuntu-22.٠٤",
        "macos-١٤",
        "windows-20٢٢",
        "windows-2022-vs٢٠٢٢",
        "brew install postgresql@١٨",
        "node-version: ٢٢",
        "node-version: \"٢\"",
        "node-version: '٢'",
        "node-version: \\\"٢\\\"",
    ]);
    assert_none(&lines);
    // `localhost` is exempted after the match, so its port is checked raw.
    assert!(raw_pins("localhost:٥٠٠٠/acme/api:2").is_empty());
    for (line, expected) in [
        ("ghcr.io/acme/api:2.1", "ghcr.io/acme/api:2.1"),
        ("ghcr.io/acme/api:pg18", "ghcr.io/acme/api:pg18"),
        ("ghcr.io:5000/acme/api:2", "ghcr.io:5000/acme/api:2"),
        ("image: postgres:18", "postgres:18"),
        ("ubuntu-22.04", "ubuntu-22.04"),
        ("brew install postgresql@18", "postgresql@18"),
        ("node-version: 22", "node-version: 22"),
    ] {
        assert_pins(line, &[expected]);
    }
    // The ASCII part before a non-ASCII digit is still a reference of its own.
    for (line, expected) in [
        ("ghcr.io/acme/api:1.٢", "ghcr.io/acme/api:1"),
        ("ghcr.io/acme/api:1.2.٣", "ghcr.io/acme/api:1.2"),
        ("ghcr.io/acme/api:pg18.٢", "ghcr.io/acme/api:pg18"),
        ("brew install openssl@3.٥", "openssl@3"),
    ] {
        assert_pins(line, &[expected]);
    }
}

#[test]
fn a_unicode_letter_is_part_of_the_context_word() {
    // `é` and `β` are letters, so they do not open `image`, `FROM`, or `brew`.
    assert_none(&owned(&[
        "éimage: postgres:18",
        "éFROM postgres:18",
        "βimage: postgres:18",
        "βFROM postgres:18",
        "βimage:ghcr.io/acme/api:v2",
        "ébrew install postgresql@18",
        "brewé install postgresql@18",
    ]));
    // An ASCII letter glued to the word still makes a different word.
    assert_none(&owned(&[
        "myimage: postgres:18",
        "xFROM postgres:18",
        "rebrew install postgresql@18",
        "brewing install postgresql@18",
    ]));
}

#[test]
fn homebrew_formulae_and_setup_keys_take_non_ascii_boundaries() {
    for (line, expected) in [
        ("brew install postgresql@18β", "postgresql@18"),
        ("brew install βpostgresql@18", "postgresql@18"),
        ("brew install openssl@3.5é", "openssl@3.5"),
        ("café brew install openssl@3.5 日本", "openssl@3.5"),
        ("βnode-version: 22", "node-version: 22"),
        ("日本 node-version: 22", "node-version: 22"),
        ("node-version: 22β", "node-version: 22"),
    ] {
        assert_pins(line, &[expected]);
    }
}

#[test]
fn an_invalid_host_label_is_not_a_host_and_its_suffix_is_not_reported() {
    // Decision: nothing is reported, not the valid suffix. A pin must start at a
    // boundary, and `-` and `.` are not one, so `acme.io/...` cannot start here.
    let mut lines = owned(&[
        "bad-.acme.io/acme/api:9",
        "bad-.acme.io/acme/api:1.2",
        "bad-.acme.io/acme/api:v2",
        "-bad.acme.io/acme/api:9",
        "-bad.acme.io/acme/api:1.2",
        "-bad.acme.io/acme/api:v2",
        "-a.b.io/x/y:1.2",
        "a-.b.io/x/y:1.2",
        "a.-b.io/x/y:1.2",
        "a.b-.io/x/y:1.2",
        "a.b.-io/x/y:1.2",
        "a.b.io-/x/y:1.2",
        "bad-.acme.io:5000/acme/api:9",
        "x -bad.acme.io/acme/api:1.2",
        "image: bad-.acme.io/acme/api:1.2",
        "image: -bad.acme.io/acme/api:1.2",
        "image:bad-.acme.io/acme/api:v2",
        "image:-bad.acme.io/acme/api:v2",
        "FROM bad-.acme.io/acme/api:1.2",
    ]);
    lines.push(format!("bad-.acme.io/acme/api@sha256:{DIGEST}"));
    lines.push(format!("-bad.acme.io/acme/api@sha256:{DIGEST}"));
    assert_none(&lines);
}

#[test]
fn hyphens_inside_a_host_label_are_valid() {
    for (line, expected) in [
        ("a-b.example.io/acme/api:9", "a-b.example.io/acme/api:9"),
        ("a-b.c-d.io/x/y:1.2", "a-b.c-d.io/x/y:1.2"),
        ("a--b.io/x/y:v2", "a--b.io/x/y:v2"),
        ("my-registry.acme.io/x/y:2", "my-registry.acme.io/x/y:2"),
        ("a.io/x/y:2", "a.io/x/y:2"),
        ("0a.9b.io/x/y:2", "0a.9b.io/x/y:2"),
        ("1.io:5000/x/y:2", "1.io:5000/x/y:2"),
        ("image:a-b.io/x/y:v5", "a-b.io/x/y:v5"),
    ] {
        assert_pins(line, &[expected]);
    }
    assert_pins(
        &format!("a-b.acme.io/x/y@sha256:{DIGEST}"),
        &[&format!("a-b.acme.io/x/y@sha256:{DIGEST}")],
    );
}

#[test]
fn localhost_and_ip_hosts_are_still_registry_hosts() {
    // `localhost:5000` is a reserved placeholder host, exempted after the match,
    // so the pattern itself is checked before the exemption.
    for line in [
        "localhost:5000/acme/api:2",
        "localhost:5000/acme/api:v2",
        "localhost:5000/acme/api:2.1",
    ] {
        assert_eq!(raw_pins(line), [line], "{line}");
    }
    let digest = format!("localhost:5000/acme/api@sha256:{DIGEST}");
    assert_eq!(raw_pins(&digest), [digest.as_str()]);
    assert!(pins("localhost:5000/acme/api:2.1").is_empty());
    assert!(raw_pins("localhost:/acme/api:2").is_empty());
    assert!(raw_pins("localhost/acme/api:2").is_empty());
    for (line, expected) in [
        ("10.0.0.5:5000/acme/api:2", "10.0.0.5:5000/acme/api:2"),
        ("10.0.0.5:5000/acme/api:v2", "10.0.0.5:5000/acme/api:v2"),
        ("192.168.1.10/acme/api:1.2", "192.168.1.10/acme/api:1.2"),
        (
            "image: 10.0.0.5:5000/acme/api:9",
            "10.0.0.5:5000/acme/api:9",
        ),
    ] {
        assert_pins(line, &[expected]);
    }
    let ip_digest = format!("10.0.0.5:5000/acme/api@sha256:{DIGEST}");
    assert_pins(&ip_digest, &[ip_digest.as_str()]);
}
