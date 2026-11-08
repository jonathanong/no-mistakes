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
fn postgres_bounded_statements_issue_2988_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_2988",
        &[(
            "issue_2988",
            repo_rel("test-cases/rules/postgres-bounded-statements/fixture"),
            rules_path("postgres-bounded-statements", "configs", "issue-2988.yml"),
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

#[test]
fn postgres_bounded_statements_issue_3100_snapshot() {
    assert_saved_fixture(
        "postgres_bounded_statements_issue_3100",
        &[(
            "issue_3100",
            repo_rel("test-cases/rules/postgres-bounded-statements/issue-3100/fixture"),
            rules_path("postgres-bounded-statements", "configs", "issue-3100.yml"),
        )],
    );
}

#[test]
fn issue_3116_recursive_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3116/fixture");
    assert_saved_fixture(
        "postgres_bounded_issue_3116_recursive",
        &[("recursive", root.clone(), root.join("recursive.yml"))],
    );
}

#[test]
fn issue_3116_correlated_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3116/fixture");
    assert_saved_fixture(
        "postgres_bounded_issue_3116_correlated",
        &[
            ("correlated", root.clone(), root.join("correlated.yml")),
            ("embedded", root.clone(), root.join("embedded.yml")),
            ("lateral_bare", root.clone(), root.join("lateral-bare.yml")),
            (
                "unknown_bare",
                root.clone(),
                root.join("correlated-unknown.yml"),
            ),
        ],
    );
}

#[test]
fn issue_3116_off_check_json_snapshot() {
    let root = repo_rel("test-cases/rules/postgres-bounded-statements/issue-3116/fixture");
    assert_saved_fixture(
        "postgres_bounded_issue_3116_off",
        &[("off", root.clone(), root.join("off.yml"))],
    );
}
