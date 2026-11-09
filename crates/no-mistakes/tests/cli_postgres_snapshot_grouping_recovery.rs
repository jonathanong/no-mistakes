use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

#[test]
fn postgres_grouping_recovery_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-grouping-not-aggregation/fixture"),
    );
    assert_saved_fixture(
        "postgres_grouping_recovery",
        &[(
            "complete_siblings_only",
            root.clone(),
            root.join("recovery.yml"),
        )],
    );
}
