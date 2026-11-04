use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

#[test]
fn postgres_bounded_statements_declared_functions_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/fixture/bounded-reuse"),
    );
    let cases = [
        (
            "declared-select-scalar",
            "bounded-reuse-declared-select-scalar.yml",
        ),
        (
            "declared-select-set",
            "bounded-reuse-declared-select-set.yml",
        ),
        (
            "declared-select-table",
            "bounded-reuse-declared-select-table.yml",
        ),
        (
            "declared-select-mixed",
            "bounded-reuse-declared-select-mixed.yml",
        ),
        (
            "declared-select-unknown",
            "bounded-reuse-declared-select-unknown.yml",
        ),
        (
            "declared-select-unknown-overload",
            "bounded-reuse-declared-select-unknown-overload.yml",
        ),
        (
            "declared-select-malformed",
            "bounded-reuse-declared-select-malformed.yml",
        ),
        (
            "declared-select-wrong-name",
            "bounded-reuse-declared-select-wrong-name.yml",
        ),
        (
            "declared-select-wrong-schema",
            "bounded-reuse-declared-select-wrong-schema.yml",
        ),
        (
            "declared-select-explicit-set",
            "bounded-reuse-declared-select-explicit-set.yml",
        ),
        (
            "declared-select-conflict",
            "bounded-reuse-declared-select-conflict.yml",
        ),
        (
            "declared-select-body-text",
            "bounded-reuse-declared-select-body-text.yml",
        ),
        (
            "declared-select-qualified",
            "bounded-reuse-declared-select-qualified.yml",
        ),
        (
            "declared-select-foreign",
            "bounded-reuse-declared-select-foreign.yml",
        ),
        (
            "declared-cte-scalar",
            "bounded-reuse-declared-cte-scalar.yml",
        ),
        ("declared-cte-set", "bounded-reuse-declared-cte-set.yml"),
        ("declared-cte-table", "bounded-reuse-declared-cte-table.yml"),
        ("declared-cte-mixed", "bounded-reuse-declared-cte-mixed.yml"),
        (
            "declared-cte-unknown",
            "bounded-reuse-declared-cte-unknown.yml",
        ),
        (
            "declared-cte-unknown-overload",
            "bounded-reuse-declared-cte-unknown-overload.yml",
        ),
        (
            "declared-cte-malformed",
            "bounded-reuse-declared-cte-malformed.yml",
        ),
        (
            "declared-cte-wrong-name",
            "bounded-reuse-declared-cte-wrong-name.yml",
        ),
        (
            "declared-cte-wrong-schema",
            "bounded-reuse-declared-cte-wrong-schema.yml",
        ),
        (
            "declared-cte-explicit-set",
            "bounded-reuse-declared-cte-explicit-set.yml",
        ),
        (
            "declared-cte-conflict",
            "bounded-reuse-declared-cte-conflict.yml",
        ),
        (
            "declared-cte-body-text",
            "bounded-reuse-declared-cte-body-text.yml",
        ),
        (
            "declared-cte-qualified",
            "bounded-reuse-declared-cte-qualified.yml",
        ),
        (
            "declared-cte-foreign",
            "bounded-reuse-declared-cte-foreign.yml",
        ),
        (
            "declared-select-set-no-limit",
            "bounded-reuse-declared-select-set-no-limit.yml",
        ),
    ]
    .map(|(label, config)| {
        (
            label,
            root.clone(),
            rules_path("postgres-bounded-statements", "configs", config),
        )
    });
    assert_saved_fixture("postgres_bounded_statements_declared_functions", &cases);
}
