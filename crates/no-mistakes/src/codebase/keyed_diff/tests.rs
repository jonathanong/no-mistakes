use super::{diff, diff_with_presence, KeyChanges, KeyFormat, KeyOptions};
use crate::codebase::lockfile;
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

mod lockfiles;
mod malformed;

const CASES: [(&str, KeyFormat, &str); 8] = [
    ("npm", KeyFormat::Npm, "json"),
    ("pnpm", KeyFormat::Pnpm, "yaml"),
    ("swift", KeyFormat::Swift, "json"),
    ("nuget", KeyFormat::Nuget, "json"),
    ("json-object", KeyFormat::Json, "json"),
    ("json-array", KeyFormat::Json, "json"),
    ("strings", KeyFormat::Strings, "strings"),
    ("resx", KeyFormat::Resx, "resx"),
];

#[derive(Serialize)]
struct Comparisons {
    edited: KeyChanges,
    moved_only: KeyChanges,
    added_file: KeyChanges,
    removed_file: KeyChanges,
    malformed: String,
}

#[test]
fn saved_keyed_diff_formats() {
    let mut report = BTreeMap::new();
    for (scenario, format, extension) in CASES {
        let fixture = |name| fixture(scenario, name, extension);
        let base = fixture("base");
        let moved = fixture("moved");
        let head = fixture("head");
        let malformed = fixture("malformed");
        let options = KeyOptions {
            format,
            array: (scenario == "json-array").then(|| "fixtures".to_string()),
            id_field: (scenario == "json-array").then(|| "id".to_string()),
        };
        let edited = diff(&options, Some(&base), Some(&head)).unwrap();
        if matches!(format, KeyFormat::Npm | KeyFormat::Pnpm) {
            assert_lockfile_parity(format, &base, &head, &edited);
        }
        report.insert(
            scenario,
            Comparisons {
                edited,
                moved_only: diff(&options, Some(&base), Some(&moved)).unwrap(),
                added_file: diff(&options, None, Some(&head)).unwrap(),
                removed_file: diff(&options, Some(&base), None).unwrap(),
                malformed: format!(
                    "{:#}",
                    diff(&options, Some(&base), Some(&malformed)).unwrap_err()
                ),
            },
        );
    }
    let rendered = serde_json::to_string_pretty(&report).unwrap();
    insta::assert_snapshot!("keyed_diff_formats", rendered);
}

fn assert_lockfile_parity(format: KeyFormat, base: &[u8], head: &[u8], edited: &KeyChanges) {
    let manager = if format == KeyFormat::Npm {
        lockfile::PackageManager::Npm
    } else {
        lockfile::PackageManager::Pnpm
    };
    let old = lockfile::parse_lockfile(manager, std::str::from_utf8(base).unwrap());
    let new = lockfile::parse_lockfile(manager, std::str::from_utf8(head).unwrap());
    let legacy = lockfile::diff(&old, &new);
    assert_eq!(legacy.changed, edited.changed);
    assert_eq!(legacy.added, edited.added);
    assert_eq!(legacy.removed, edited.removed);
}

#[test]
fn keyed_diff_preserves_ids_for_identical_inputs_and_rejects_identically_malformed_inputs() {
    let options = KeyOptions {
        format: KeyFormat::Strings,
        array: None,
        id_field: None,
    };
    let base = fixture("strings", "base", "strings");
    let comments = fixture("strings", "comments", "strings");
    let (unchanged, [left, right]) =
        diff_with_presence(&options, Some(&base), Some(&base)).unwrap();
    assert!(unchanged.is_empty());
    assert_eq!(left, ["alpha", "moving", "removed"]);
    assert_eq!(right, left);
    assert!(diff(&options, Some(&base), Some(&comments))
        .unwrap()
        .is_empty());

    let invalid = fixture("strings", "invalid-unicode", "strings");
    let error = diff(&options, Some(&invalid), Some(&invalid)).unwrap_err();
    assert!(format!("{error:#}").contains("invalid Unicode escape"));
}

#[test]
fn keyed_diff_selects_nested_arrays_by_dotted_path_or_pointer() {
    let source = fixture("json-array", "nested", "json");
    for path in ["catalog.fixtures", "/catalog/fixtures"] {
        let options = KeyOptions {
            format: KeyFormat::Json,
            array: Some(path.to_string()),
            id_field: Some("id".to_string()),
        };
        assert_eq!(
            diff(&options, None, Some(&source)).unwrap().added,
            ["alpha"]
        );
    }
}

#[test]
fn keyed_diff_decodes_saved_catalogs_as_utf16_with_both_byte_orders() {
    for (name, format, extension) in CASES
        .into_iter()
        .filter(|(_, format, _)| matches!(format, KeyFormat::Strings | KeyFormat::Resx))
    {
        let options = KeyOptions {
            format,
            array: None,
            id_field: None,
        };
        let base = fixture(name, "base", extension);
        let head = fixture(name, "head", extension);
        let expected = diff(&options, Some(&base), Some(&head)).unwrap();
        for big_endian in [false, true] {
            let encode = |source: &[u8]| {
                let mut bytes = if big_endian {
                    vec![0xfe, 0xff]
                } else {
                    vec![0xff, 0xfe]
                };
                for unit in std::str::from_utf8(source).unwrap().encode_utf16() {
                    bytes.extend(if big_endian {
                        unit.to_be_bytes()
                    } else {
                        unit.to_le_bytes()
                    });
                }
                bytes
            };
            let encoded_base = encode(&base);
            let mut encoded_head = encode(&head);
            assert_eq!(
                diff(&options, Some(&encoded_base), Some(&encoded_head)).unwrap(),
                expected
            );
            let truncated = &encoded_head[..encoded_head.len() - 1];
            assert!(format!(
                "{:#}",
                diff(&options, Some(&encoded_base), Some(truncated)).unwrap_err()
            )
            .contains("incomplete UTF-16 code unit"));
            encoded_head.extend(if big_endian {
                0xd800_u16.to_be_bytes()
            } else {
                0xd800_u16.to_le_bytes()
            });
            assert!(format!(
                "{:#}",
                diff(&options, Some(&encoded_base), Some(&encoded_head)).unwrap_err()
            )
            .contains("invalid UTF-16 text"));
        }
    }
}

fn fixture(scenario: &str, name: &str, extension: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/keyed-diff")
        .join(scenario)
        .join("fixture")
        .join(format!("{name}.{extension}"));
    std::fs::read(path).unwrap()
}
