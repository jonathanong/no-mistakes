use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "no-test-git-sha";

fn review_sha_root() -> std::path::PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/repository-guard-review/no-test-git-sha"),
    )
}

#[test]
fn no_test_git_sha_json_matches_snapshot() {
    let tests_include = rules_path(RULE, "configs", "tests-include.yml");
    let fail = rules_path(RULE, "fixture", "fail");
    let pass = rules_path(RULE, "fixture", "pass");
    assert_saved_fixture(
        "no_test_git_sha",
        &[
            (
                "allowed_context",
                fail.clone(),
                rules_path(RULE, "configs", "allowed-synthetic.yml"),
            ),
            ("both_lines", fail.clone(), tests_include.clone()),
            ("tests_include", pass.clone(), tests_include.clone()),
            (
                "src_include",
                pass,
                rules_path(RULE, "configs", "src-include.yml"),
            ),
            (
                "exclude_example",
                fail,
                rules_path(RULE, "configs", "exclude-example.yml"),
            ),
            ("hex_boundaries", review_sha_root(), tests_include),
        ],
    );
}
