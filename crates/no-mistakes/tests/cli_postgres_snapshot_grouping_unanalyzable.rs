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
    assert_result(&snapshots[policy], diagnostic_expected);
}

fn assert_result(result: &serde_json::Value, diagnostic_expected: bool) {
    let report = &result["report"];
    assert_eq!(result["exitCode"], 0, "{report}");
    assert_eq!(report["rules"], serde_json::json!([]), "{report}");
    let advisories = report["advisories"].as_array().unwrap();
    assert!(
        advisories.iter().any(|finding| {
            finding["rule"] == RULE
                && finding["target"] == "distinct-over-unique-key"
                && finding["line"] == 4
        }),
        "the configured executor must retain its proven grouping advisory: {report}"
    );
    let diagnostics: Vec<_> = advisories
        .iter()
        .filter(|finding| finding["rule"] == RULE && finding["line"] == 5)
        .collect();
    assert_eq!(
        diagnostics.len(),
        usize::from(diagnostic_expected),
        "{report}"
    );
    assert_eq!(advisories.len(), 1 + diagnostics.len(), "{report}");
}

#[test]
fn postgres_grouping_unanalyzable_policy_parity_check_json_snapshot() {
    let root = rules_path(RULE, "fixture", "unanalyzable");
    let policies = [
        ("default", true),
        ("advise", true),
        ("ignore", false),
        ("rule-fail", true),
        ("shared-report", true),
        ("shared-sql-report", true),
        ("project-fail", true),
        ("rule-report", true),
        ("rule-ignore-over-fail", false),
        ("shared-ignore-rule-fail", true),
        ("project-fail-rule-ignore", false),
    ];
    let cases: Vec<_> = policies
        .iter()
        .map(|(name, _)| (*name, root.clone(), root.join(format!("{name}.yml"))))
        .collect();
    let reports = assert_saved_fixture_report(
        "postgres_grouping_unanalyzable_policy_parity",
        &cases,
        &[],
        &[],
    );
    for (name, diagnostic_expected) in policies {
        assert_result(&reports[name], diagnostic_expected);
        let expected = if diagnostic_expected {
            "advise"
        } else {
            "ignore"
        };
        assert_eq!(reports[name], reports[expected], "{name}: {reports}");
    }
}
