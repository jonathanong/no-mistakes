use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-test-timeout-literals";

#[test]
fn github_actions_test_timeout_literals_json_matches_snapshot() {
    let default = rules_path(RULE, "configs", "default.yml");
    assert_saved_fixture(
        "github_actions_test_timeout_literals",
        &[
            // Four restated literals, including the yaml `toContain` target.
            (
                "restated_literals",
                rules_path(RULE, "fixture", "fail"),
                default.clone(),
            ),
            // `src/other.test.mts` has a literal and is outside the default include.
            ("pass", rules_path(RULE, "fixture", "pass"), default),
            (
                "allow_matching",
                rules_path(RULE, "fixture", "allow"),
                rules_path(RULE, "configs", "allow-matching.yml"),
            ),
            // The allow text is absent, so the stale pin is the finding.
            (
                "stale_allow",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "allow-stale.yml"),
            ),
            (
                "allow_unscanned_file",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "allow-unscanned.yml"),
            ),
            // Scans ci.test.mts only; ci.test.ts stays out of this include.
            (
                "custom_include",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "include-ci-test-mts.yml"),
            ),
            // Without this include, fail reports the four restated literals.
            (
                "include_without_matches",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "include-no-match.yml"),
            ),
        ],
    );
}
