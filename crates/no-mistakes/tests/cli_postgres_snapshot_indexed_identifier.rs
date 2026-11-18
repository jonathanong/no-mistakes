use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_indexed_identifier_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3598",
    );
    let report = assert_saved_fixture_report(
        "postgres_indexed_identifier",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = report["max"]["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 26);
    assert!(findings
        .iter()
        .all(|finding| finding["file"] == "negative.ts"
            || finding["file"] == "mutations.ts"
            || finding["file"] == "class-shadow.ts"));
    assert_eq!(report["max"]["report"]["warnings"], serde_json::json!([]));
    assert_eq!(report["max"]["report"]["advisories"], serde_json::json!([]));
}

#[test]
fn postgres_imported_tuple_identifier_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3631",
    )
    .join("app");
    let report = assert_saved_fixture_report(
        "postgres_imported_tuple_identifier",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = report["max"]["report"]["rules"].as_array().unwrap();
    let mut files = std::collections::BTreeMap::new();
    for finding in findings {
        *files.entry(finding["file"].as_str().unwrap()).or_insert(0) += 1;
    }
    assert_eq!(
        files,
        [
            ("boundary.mts", 5),
            ("missing.mts", 2),
            ("negative.mts", 9),
            ("role-collision.mts", 2),
        ]
        .into()
    );
    assert_eq!(report["max"]["report"]["warnings"], serde_json::json!([]));
    assert_eq!(report["max"]["report"]["advisories"], serde_json::json!([]));
}

#[test]
fn postgres_tuple_identifier_composition_parity_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3631-parity",
    );
    let report = assert_saved_fixture_report(
        "postgres_tuple_identifier_composition_parity",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = report["max"]["report"]["rules"].as_array().unwrap();
    let mut locations = findings
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    locations.sort_unstable();
    let expected = ["imported.mts", "local.mts"]
        .into_iter()
        .flat_map(|file| [9, 10, 17, 18, 24, 25].map(|line| (file, line)))
        .collect::<Vec<_>>();
    assert_eq!(locations, expected);
    assert_eq!(report["max"]["report"]["warnings"], serde_json::json!([]));
    assert_eq!(report["max"]["report"]["advisories"], serde_json::json!([]));
}
