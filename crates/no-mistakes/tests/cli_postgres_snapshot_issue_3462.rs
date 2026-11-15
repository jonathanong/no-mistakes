use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

#[test]
fn postgres_bounded_issue_3462_check_json_snapshot() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-bounded-statements/issue-3462"),
    );
    assert_saved_fixture(
        "postgres_bounded_issue_3462",
        &[("view-table-name", root.clone(), root.join("report.yml"))],
    );
}
