use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "postgres-grouping-not-aggregation";

#[test]
fn postgres_grouping_unanalyzable_ignore_check_json_snapshot() {
    assert_policy("ignore", false);
}

#[test]
fn postgres_grouping_unanalyzable_advise_check_json_snapshot() {
    assert_policy("advise", true);
}

#[test]
fn postgres_grouping_unanalyzable_default_check_json_snapshot() {
    assert_policy("default", true);
}

fn assert_policy(policy: &str, diagnostic_expected: bool) {
    let root = rules_path(RULE, "fixture", "unanalyzable");
    let snapshots = assert_saved_fixture_report(
        &format!("postgres_grouping_unanalyzable_{policy}"),
        &[(policy, root.clone(), root.join(format!("{policy}.yml")))],
        &[],
        &[],
    );
    let result = &snapshots[policy];
    let report = &result["report"];
    assert_eq!(result["exitCode"], 0, "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let advisories = report["advisories"].as_array().unwrap();
    assert!(
        advisories.iter().any(|finding| {
            finding["rule"] == RULE
                && finding["target"] == "distinct-over-unique-key"
                && finding["line"] == 8
        }),
        "the configured executor must retain its proven grouping advisory: {report}"
    );
    let diagnostics: Vec<_> = advisories
        .iter()
        .filter(|finding| finding["rule"] == RULE && finding["line"] == 9)
        .collect();
    assert_eq!(
        diagnostics.len(),
        usize::from(diagnostic_expected),
        "{report}"
    );
    assert_eq!(advisories.len(), 1 + diagnostics.len(), "{report}");
}
