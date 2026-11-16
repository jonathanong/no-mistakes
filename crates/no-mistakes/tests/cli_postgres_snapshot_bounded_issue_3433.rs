use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_bounded_statements_issue_3433_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3433/fixture");
    assert_saved_fixture(
        "postgres_bounded_statements_issue_3433",
        &[("issue_3433", root.clone(), root.join(".no-mistakes.yml"))],
    );
}
