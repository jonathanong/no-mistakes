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
    let mut reported = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| {
            let message = finding["message"].as_str().unwrap();
            let kind = if message.contains("must start with a /* name */ annotation") {
                "missing"
            } else if message.contains("leading SQL is unanalyzable") {
                "unanalyzable"
            } else {
                message
            };
            (finding["sourceLine"].as_str().unwrap(), kind)
        })
        .collect::<Vec<_>>();
    reported.sort_unstable();
    assert_eq!(
        reported,
        vec![
            (
                "export const missing = (limit: number) => queryIdPage(limit, sql`SELECT id FROM widgets`, statement => read(statement));",
                "missing",
            ),
            (
                "export const opaque = (limit: number, value: SqlStatement) => queryIdPage(limit, value, statement => read(statement));",
                "unanalyzable",
            ),
            (
                "export const reassigned = (limit: number) => queryItemsReassign(limit, sql`/* reassigned */ SELECT id FROM orders WHERE TRUE`, statement => read(statement));",
                "unanalyzable",
            ),
        ]
    );
}
