use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_opaque_functions_check_json_snapshot() {
    let root = rules_path("postgres-ddl-unanalyzable", "fixture", "opaque-functions");
    let cases: Vec<_> = [
        "execute-using",
        "execute-into",
        "conflict-row",
        "top-level-conflict",
        "body-ddl",
        "nested-stored",
        "report",
    ]
    .into_iter()
    .map(|label| (label, root.clone(), root.join(format!("{label}.yml"))))
    .collect();
    assert_saved_fixture("postgres_opaque_functions", &cases);
}
