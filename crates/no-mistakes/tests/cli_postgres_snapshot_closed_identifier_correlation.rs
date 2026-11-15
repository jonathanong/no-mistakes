use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_closed_identifier_correlation_check_json_snapshot() {
    let root = rules_path(
        "postgres-bounded-statements",
        "closed-identifier-correlation",
        "fixture",
    );
    let cases = [("correlation", root.clone(), root.join(".no-mistakes.yml"))];
    let report =
        assert_saved_fixture_report("postgres_closed_identifier_correlation", &cases, &[], &[]);
    let rows = report["correlation"]["report"]["rules"].as_array().unwrap();
    assert_eq!(rows.len(), 4, "{rows:?}");
    let row = |needle: &str| {
        rows.iter()
            .find(|row| {
                row["sourceLine"]
                    .as_str()
                    .is_some_and(|line| line.contains(needle))
            })
            .unwrap_or_else(|| panic!("missing {needle}: {rows:?}"))
    };
    assert_eq!(row("column(other)")["target"], "table:posts, table:users");
    assert_eq!(row("kind === \"user\"")["target"], "unanalyzable");
    assert_eq!(row("column(k)")["target"], "unanalyzable");
    assert_eq!(row("column(\"missing\")")["target"], "unanalyzable");
}
