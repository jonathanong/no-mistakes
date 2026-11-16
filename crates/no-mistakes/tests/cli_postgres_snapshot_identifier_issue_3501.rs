use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_identifier_issue_3501_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3501",
    );
    let report = assert_saved_fixture_report(
        "postgres_identifier_issue_3501",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );
    let findings = report["max"]["report"]["rules"].as_array().unwrap();
    let actual: std::collections::BTreeSet<_> = findings
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    let expected = [
        ("negative/assertion-callback.ts", vec![9]),
        ("negative/callback-controls.ts", vec![8, 9, 14]),
        ("negative/callbacks.ts", vec![8, 9, 12]),
        (
            "negative/cast-aliases.ts",
            vec![7, 11, 15, 19, 23, 28, 32, 37],
        ),
        ("negative/cyclic-type.ts", vec![5]),
        ("negative/duplicate-type.ts", vec![5]),
        ("negative/guards.ts", vec![11, 15, 21, 26]),
        ("negative/inline-arrays.ts", vec![9, 10, 11, 12, 13, 14]),
        ("negative/literal-unions.ts", vec![4, 8, 12, 17, 21, 26, 30]),
        ("negative/maps.ts", vec![11, 12, 13, 14]),
        ("negative/metadata.ts", vec![8, 14, 20, 26]),
        ("negative/type-scope.ts", vec![5, 11, 16, 22]),
        (
            "negative/nonrequired-literal-unions.ts",
            vec![6, 10, 14, 18],
        ),
    ]
    .into_iter()
    .flat_map(|(file, lines)| lines.into_iter().map(move |line| (file, line)))
    .collect();
    assert_eq!(actual, expected);
    assert_eq!(findings.len(), actual.len());
    for category in ["warnings", "advisories"] {
        assert_eq!(report["max"]["report"][category], serde_json::json!([]));
    }
}
