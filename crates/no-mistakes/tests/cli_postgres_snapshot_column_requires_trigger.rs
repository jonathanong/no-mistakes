use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_column_requires_trigger_check_json_snapshot() {
    let rule = "postgres-column-requires-trigger";
    let root = repo_rel("fixtures/postgres/column-requires-trigger");
    let cases = [
        // Messages must not contain table:orders:, table:invoices:, table:events:, or table:order_events:.
        ("touch", "touch.yml"),
        // Messages must not contain table:orders:.
        ("shared_function", "shared-function.yml"),
        ("include_misses_catalog", "include-miss.yml"),
        ("two_requirements", "two-requirements.yml"),
        // Messages must not contain partial_orders.
        ("allow_column_list", "allow-column-list.yml"),
        ("instead_of_statement", "instead-of.yml"),
        ("empty_requirements", "empty.yml"),
        // Messages must not contain table:import_rows:.
        ("allow_stale", "allow-stale.yml"),
        ("after_insert", "after-insert.yml"),
        ("after_truncate", "after-truncate.yml"),
        ("folded_function_names", "folded-names.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), rules_path(rule, "configs", config)));
    assert_saved_fixture("postgres_column_requires_trigger", &cases);
}
