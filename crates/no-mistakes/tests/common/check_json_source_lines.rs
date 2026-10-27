//! Adds the trimmed source line to each normalized finding.
//!
//! A snapshot that stores only `file` and `line` cannot show a rule pointing
//! at the wrong line. The text is read from the saved fixture (or the checkout
//! for `<repo>/` config findings) after path normalization, folded to LF, and
//! trimmed, so it is identical on every operating system.

use serde_json::{Map, Value};
use std::path::Path;

pub fn add_source_lines(value: &mut Value, root: &Path, repo: &Path) {
    match value {
        Value::Array(items) => {
            for item in items {
                add_source_lines(item, root, repo);
            }
        }
        Value::Object(map) => {
            if let Some(text) = finding_source(map, root, repo) {
                map.insert("sourceLine".to_string(), Value::String(text));
            }
            for child in map.values_mut() {
                add_source_lines(child, root, repo);
            }
        }
        _ => {}
    }
}

fn finding_source(map: &Map<String, Value>, root: &Path, repo: &Path) -> Option<String> {
    let file = map.get("file")?.as_str()?;
    let line = map.get("line")?.as_u64()?;
    if Path::new(file).is_absolute() {
        return None;
    }
    let path = match file.strip_prefix("<repo>/") {
        Some(rest) => repo.join(rest),
        None => root.join(file),
    };
    let text = std::fs::read_to_string(path).ok()?;
    source_line(&text, line)
}

pub fn source_line(text: &str, line: u64) -> Option<String> {
    let index = usize::try_from(line.checked_sub(1)?).ok()?;
    // CRLF, CR, LF, U+2028 and U+2029 each end a line, matching how the
    // TS/JS parser numbers lines, so a CR-only fixture still lines up.
    let folded = text.replace("\r\n", "\n");
    folded
        .split(['\n', '\r', '\u{2028}', '\u{2029}'])
        .nth(index)
        .map(|row| row.trim().to_string())
}

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
}
