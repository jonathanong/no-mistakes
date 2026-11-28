use super::{diff, KeyChanges, KeyFormat, KeyOptions};
use crate::codebase::lockfile;
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

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

fn fixture(scenario: &str, name: &str, extension: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/keyed-diff")
        .join(scenario)
        .join("fixture")
        .join(format!("{name}.{extension}"));
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
