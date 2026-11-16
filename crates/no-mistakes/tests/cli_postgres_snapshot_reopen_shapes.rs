use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn case_root(rule: &str, name: &str) -> PathBuf {
    let mut path = rules_path(rule, name, "src");
    path.pop();
    path
}

#[test]
fn postgres_reopen_shapes_check_json_snapshot() {
    assert_saved_fixture(
        "postgres_reopen_shapes",
        &[
            (
                "cross_module_claim",
                case_root("postgres-lock-ordering", "cross-module-claim"),
                rules_path("postgres-lock-ordering", "cross-module-claim", "config.yml"),
            ),
            (
                "statement_append",
                case_root("postgres-lock-ordering", "statement-append"),
                rules_path("postgres-lock-ordering", "statement-append", "config.yml"),
            ),
            (
                "raw_column_append",
                case_root("postgres-required-predicates", "raw-column-append"),
                rules_path(
                    "postgres-required-predicates",
                    "raw-column-append",
                    "config.yml",
                ),
            ),
            (
                "imported_limit",
                case_root("postgres-conflict-ordering", "imported-limit"),
                rules_path("postgres-conflict-ordering", "imported-limit", "config.yml"),
            ),
        ],
    );
}
