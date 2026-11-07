use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

fn snapshot(shape: &str) {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/issue-3114"),
    );
    assert_saved_fixture(
        &format!("postgres_bounded_issue_3114_{shape}"),
        &[(shape, root.clone(), root.join(format!("{shape}.yml")))],
    );
}

#[test]
fn postgres_bounded_issue_3114_drivers_check_json_snapshot() {
    snapshot("drivers");
}

#[test]
fn postgres_bounded_issue_3114_cte_writes_check_json_snapshot() {
    snapshot("cte-writes");
}

#[test]
fn postgres_bounded_issue_3114_unnest_check_json_snapshot() {
    snapshot("unnest");
}

#[test]
fn postgres_bounded_issue_3114_unqualified_check_json_snapshot() {
    snapshot("unqualified");
}

#[test]
fn postgres_bounded_issue_3114_indexes_check_json_snapshot() {
    snapshot("indexes");
}
