use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_branchless_append_destructure_check_json_snapshot() {
    const RULE: &str = "postgres-require-query-annotation";
    let root = rules_path(RULE, "fixture", "imported-callback-append-branchless");
    let config = rules_path(
        RULE,
        "configs",
        "imported-callback-append-branchless-report.yml",
    );
    let reports = assert_saved_fixture_report(
        "postgres_branchless_append_destructure",
        &[("report", root, config)],
        &[],
        &[],
    );
    let report = &reports["report"]["report"];
    assert_eq!(reports["report"]["exitCode"], 1);
    assert_eq!(report["warnings"], serde_json::json!([]));
    let mut sites = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    sites.sort_unstable();
    assert_eq!(sites, vec![("callers.mts", 8), ("callers.mts", 9)]);
    let messages = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["message"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("must start with a /* name */ annotation")),
        "{messages:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("leading SQL is unanalyzable")),
        "{messages:?}"
    );
}
