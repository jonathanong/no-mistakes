use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-cache-key-policy";

#[test]
fn github_actions_cache_variable_fd_redirects_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_cache_fd_redirects",
        &[(
            "fd-redirects",
            rules_path(RULE, "fixture", "fd-redirects"),
            rules_path(RULE, "configs", "policy.yml"),
        )],
    );
}
