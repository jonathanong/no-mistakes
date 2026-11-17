use super::check_json_snapshot::{assert_saved_fixture_with_env_and_args, rules_path};

#[test]
fn postgres_issue_3426_scalar_nullish_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "issue-3426",
        "scalar-nullish",
    );
    assert_saved_fixture_with_env_and_args(
        "postgres_issue_3426_scalar_nullish",
        &[("scalar_nullish", root.clone(), root.join("config.yml"))],
        &[],
        &["--include-sql-verdicts"],
    );
}

#[test]
fn postgres_issue_3426_branch_append_check_json_snapshot() {
    let root = rules_path("postgres-lock-ordering", "issue-3426", "branch-append");
    assert_saved_fixture_with_env_and_args(
        "postgres_issue_3426_branch_append",
        &[("branch_append", root.clone(), root.join("config.yml"))],
        &[],
        &["--include-sql-verdicts"],
    );
}

#[test]
fn postgres_issue_3426_recursive_comment_binary_check_json_snapshot() {
    let root = rules_path(
        "postgres-bounded-statements",
        "issue-3426",
        "recursive-comment-binary",
    );
    assert_saved_fixture_with_env_and_args(
        "postgres_issue_3426_recursive_comment_binary",
        &[
            ("without_key", root.clone(), root.join("config-no-key.yml")),
            ("with_key", root.clone(), root.join("config.yml")),
        ],
        &[],
        &["--include-sql-verdicts"],
    );
}

#[test]
fn postgres_issue_3426_scalar_nullish_configured_tag_check_json_snapshot() {
    let root = rules_path(
        "postgres-no-test-state-mutation",
        "issue-3426",
        "scalar-nullish-configured-tag",
    );
    assert_saved_fixture_with_env_and_args(
        "postgres_issue_3426_scalar_nullish_configured_tag",
        &[("configured_tag", root.clone(), root.join("config.yml"))],
        &[],
        &["--include-sql-verdicts"],
    );
}
