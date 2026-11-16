use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report};
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
fn postgres_bounded_domain_rows_tail_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-tail"),
    );
    assert_saved_fixture(
        "postgres_bounded_domain_rows_tail",
        &[
            ("unset", root.clone(), root.join("tail-unset.yml")),
            ("rows_5000", root.clone(), root.join("tail-5000.yml")),
        ],
    );
}

#[test]
fn postgres_bounded_domain_rows_alias_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-alias"),
    );
    assert_saved_fixture(
        "postgres_bounded_domain_rows_alias",
        &[
            ("unset", root.clone(), root.join("alias-unset.yml")),
            ("rows_5", root.clone(), root.join("alias-5.yml")),
        ],
    );
}

#[test]
fn postgres_bounded_domain_rows_precedence_matches_with_the_cap_unset() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-precedence"),
    );
    let report = assert_saved_fixture_report(
        "postgres_bounded_domain_rows_precedence",
        &[
            ("unset", root.clone(), root.join("precedence-unset.yml")),
            ("rows_5000", root.clone(), root.join("precedence-5000.yml")),
        ],
        &[],
        &[],
    );
    assert_eq!(report["unset"], report["rows_5000"]);
}

#[test]
fn postgres_bounded_domain_rows_redundant_allow_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-redundant-allow"),
    );
    assert_saved_fixture(
        "postgres_bounded_domain_rows_redundant_allow",
        &[
            ("unset", root.clone(), root.join("redundant-unset.yml")),
            ("rows_5000", root.clone(), root.join("redundant-5000.yml")),
        ],
    );
}

#[test]
fn postgres_bounded_domain_rows_body_check_json_snapshot() {
    let root = root();
    assert_saved_fixture(
        "postgres_bounded_domain_rows_body",
        &[
            (
                "unset",
                root.clone(),
                root.join("domain-rows-body-unset.yml"),
            ),
            ("rows_5", root.clone(), root.join("domain-rows-body-5.yml")),
        ],
    );
}

#[test]
fn postgres_bounded_domain_rows_alias_shapes_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/domain-rows-alias-shapes"),
    );
    assert_saved_fixture(
        "postgres_bounded_domain_rows_alias_shapes",
        &[
            ("unset", root.clone(), root.join("shapes-unset.yml")),
            ("rows_5", root.clone(), root.join("shapes-5.yml")),
        ],
    );
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
