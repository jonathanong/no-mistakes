use crate::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_correlated_member_guards_check_json_snapshot() {
    let rule = "postgres-require-query-annotation";
    let root = rules_path(rule, "variants", "correlated-member-guards");
    let document = assert_saved_fixture_report(
        "postgres_correlated_member_guards",
        &[("max_advise", root.clone(), root.join(".no-mistakes.yml"))],
        &[],
        &[],
    );

    let result = &document["max_advise"];
    assert_eq!(result["exitCode"], 0);
    let report = &result["report"];
    assert_eq!(report["rules"], serde_json::json!([]));
    assert_eq!(report["warnings"], serde_json::json!([]));
    let mut lines = report["advisories"]
        .as_array()
        .unwrap()
        .iter()
        .map(|advisory| {
            assert_eq!(advisory["rule"], rule);
            assert_eq!(advisory["target"], "sql-composition");
            assert_eq!(advisory["file"], "src/query.ts");
            advisory["line"].as_u64().unwrap()
        })
        .collect::<Vec<_>>();
    lines.sort_unstable();
    assert_eq!(lines, [55, 74, 90, 105, 121], "{report}");
}
