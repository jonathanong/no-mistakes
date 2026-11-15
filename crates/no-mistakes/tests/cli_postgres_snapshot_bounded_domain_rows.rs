use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

fn root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows"),
    )
}

#[test]
fn postgres_bounded_domain_rows_check_json_snapshot() {
    let root = root();
    assert_saved_fixture(
        "postgres_bounded_domain_rows",
        &[
            ("unset", root.clone(), root.join("domain-rows-unset.yml")),
            ("rows_5000", root.clone(), root.join("domain-rows-5000.yml")),
            ("rows_50", root.clone(), root.join("domain-rows-50.yml")),
        ],
    );
}

fn group_order() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-group-order"),
    )
}

#[test]
fn postgres_bounded_domain_rows_group_order_check_json_snapshot() {
    let root = group_order();
    assert_saved_fixture(
        "postgres_bounded_domain_rows_group_order",
        &[
            ("unset", root.clone(), root.join("group-order-unset.yml")),
            ("rows_5000", root.clone(), root.join("group-order-5000.yml")),
            ("rows_999", root.clone(), root.join("group-order-999.yml")),
        ],
    );
}
