use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_aliased_identifier_guard_transaction_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-aliased-guards",
    );
    assert_saved_fixture_clean(
        "postgres_aliased_identifier_guard_transaction",
        &[("clean", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn postgres_aliased_identifier_guard_unchecked_union_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-aliased-guards",
    );
    assert_saved_fixture(
        "postgres_aliased_identifier_guard_unchecked_union",
        &[(
            "reported",
            root.clone(),
            root.join(".no-mistakes-negative.yml"),
        )],
    );
}
