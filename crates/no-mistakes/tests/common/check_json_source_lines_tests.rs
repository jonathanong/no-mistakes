//! Unit tests for the `sourceLine` helper; linked into exactly one suite so
//! they do not re-run wherever `check_json_snapshot` is included.

use crate::check_json_snapshot::source_lines::{add_source_lines, source_line};
use std::path::Path;

#[test]
fn source_line_is_trimmed_lf_folded_and_one_based() {
    let text = "first\r\n  second\t\r\nthird";
    assert_eq!(source_line(text, 1).as_deref(), Some("first"));
    assert_eq!(source_line(text, 2).as_deref(), Some("second"));
    assert_eq!(source_line(text, 3).as_deref(), Some("third"));
    let mixed = "a\rb\u{2028}c\r\nd\ne";
    for (line, expected) in ["a", "b", "c", "d", "e"].iter().enumerate() {
        assert_eq!(
            source_line(mixed, line as u64 + 1).as_deref(),
            Some(*expected)
        );
    }
    assert_eq!(source_line(text, 0), None);
    assert_eq!(source_line(text, 4), None);
}

#[test]
fn only_findings_with_a_readable_file_and_line_gain_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut value = serde_json::json!({
        "rules": [
            {"file": "Cargo.toml", "line": 1},
            {"file": "missing.txt", "line": 1},
            {"file": "Cargo.toml", "line": 99999},
            {"file": "Cargo.toml"},
            {"file": "/Cargo.toml", "line": 1},
            {"file": "<repo>/crates/no-mistakes/Cargo.toml", "line": 1},
            {"file": "../../Cargo.toml", "line": 1},
            {"file": "<repo>/../outside.txt", "line": 1},
        ]
    });
    add_source_lines(&mut value, root, root.ancestors().nth(2).unwrap());
    let rows = value["rules"].as_array().unwrap();
    assert_eq!(rows[0]["sourceLine"], "[package]");
    assert!(rows[1].get("sourceLine").is_none());
    assert!(rows[2].get("sourceLine").is_none());
    assert!(rows[3].get("sourceLine").is_none());
    assert!(
        rows[4].get("sourceLine").is_none(),
        "absolute paths are never read"
    );
    assert_eq!(rows[5]["sourceLine"], "[package]");
    assert!(
        rows[6].get("sourceLine").is_none(),
        "a path escaping the fixture root is never read"
    );
    assert!(rows[7].get("sourceLine").is_none());
}
