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
fn postgres_bounded_statements_issue_3397_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3397");
    assert_saved_fixture(
        "postgres_bounded_statements_issue_3397",
        &[
            ("function_calls", root.clone(), root.join("valid.yml")),
            ("named_assignment", root.clone(), root.join("uuid.yml")),
        ],
    );
}

#[test]
fn postgres_bounded_statements_issue_3397_reopen_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3397-reopen");
    assert_saved_fixture(
        "postgres_bounded_statements_issue_3397_reopen",
        &[("reopen", root.clone(), root.join("valid.yml"))],
    );
}
