use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "markdown-eval-tests";

#[test]
fn markdown_eval_tests_json_matches_snapshot() {
    let included = rules_path(RULE, "configs", "included.yml");
    assert_saved_fixture(
        "markdown_eval_tests",
        &[
            (
                "eval_spawn",
                rules_path(RULE, "fixture", "fail"),
                included.clone(),
            ),
            ("spawn_free", rules_path(RULE, "fixture", "pass"), included),
            (
                "exact_allow",
                rules_path(RULE, "fixture", "allow"),
                rules_path(RULE, "configs", "allow.yml"),
            ),
            (
                "stale_allow",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "stale-allow.yml"),
            ),
        ],
    );
}
