use super::check_json_clean::assert_saved_fixture_clean;
use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_generated_identifier_returns_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-unbounded-sql-identifier",
        "fixture",
        "issue-3073-generated",
    );
    assert_saved_fixture(
        "postgres_generated_identifier_returns",
        &[("max", root.clone(), root.join(".no-mistakes.yml"))],
    );
    assert_saved_fixture_clean(
        "postgres_generated_identifier_returns_clean",
        &[("clean", root.clone(), root.join(".no-mistakes-clean.yml"))],
    );
}
