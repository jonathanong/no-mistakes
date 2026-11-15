use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_closed_identifiers_check_json_snapshot() {
    let mut root = rules_path("postgres-closed-identifiers", "fixture", "placeholder");
    root.pop();
    let cases = [
        ("positive", root.clone(), root.join("positive.yml")),
        (
            "mutations_report",
            root.clone(),
            root.join("mutations-report.yml"),
        ),
        (
            "mutations_advise",
            root.clone(),
            root.join("mutations-advise.yml"),
        ),
        (
            "mutations_ignore",
            root.clone(),
            root.join("mutations-ignore.yml"),
        ),
        ("opaque", root.clone(), root.join("opaque.yml")),
        (
            "assertions_not_configured",
            root.clone(),
            root.join("assertions-not-configured.yml"),
        ),
        (
            "raw_tags_not_configured",
            root.clone(),
            root.join("raw-tags-not-configured.yml"),
        ),
    ];
    let report = assert_saved_fixture_report("postgres_closed_identifiers", &cases, &[], &[]);
    for category in ["rules", "warnings", "advisories"] {
        assert_eq!(
            report["positive"]["report"][category]
                .as_array()
                .unwrap()
                .len(),
            0,
            "finite positive cases must be fully analyzed: {report}"
        );
    }
    for mode in ["mutations_report", "mutations_advise", "mutations_ignore"] {
        let rows = report[mode]["report"]["rules"].as_array().unwrap();
        for target in ["DROP TABLE", "pg_sleep", "table:users", "table:posts"] {
            assert!(
                rows.iter().any(|row| {
                    row["target"]
                        .as_str()
                        .is_some_and(|value| value.split(", ").any(|part| part == target))
                }),
                "{mode} lost a definite mutation or unbounded variant for {target}: {rows:?}"
            );
        }
        assert!(
            rows.iter().any(|row| {
                row["target"] == "DROP TABLE"
                    && row["sourceLine"]
                        .as_str()
                        .is_some_and(|line| line.contains("mixedPoisoned"))
            }),
            "{mode} lost the known mutation beside an opaque branch: {rows:?}"
        );
        assert!(!rows.iter().any(|row| {
            row["sourceLine"]
                .as_str()
                .is_some_and(|line| line.contains("suppressedDrop"))
        }));
    }
    let opaque = report["opaque"]["report"]["rules"].as_array().unwrap();
    assert_eq!(
        opaque
            .iter()
            .filter(|row| row["target"] == "unanalyzable-sql" || row["target"] == "unanalyzable")
            .count(),
        42,
        "every unknown source must remain opaque for both rules: {opaque:?}"
    );
}
