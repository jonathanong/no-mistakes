use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_bounded_statements_issue_2961_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_2961",
        &[(
            "issue_2961",
            repo_rel("test-cases/rules/postgres-bounded-statements/fixture"),
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "issue-2961-bounded-false-positives.yml",
            ),
        )],
    );
}

#[test]
fn postgres_bounded_statements_issue_2961_remainder_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_2961_remainder",
        &[(
            "issue_2961_remainder",
            repo_rel("test-cases/rules/postgres-bounded-statements/fixture"),
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "issue-2961-remainder.yml",
            ),
        )],
    );
}
