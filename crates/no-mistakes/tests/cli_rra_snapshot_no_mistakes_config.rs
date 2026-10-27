use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "no-mistakes-config";

#[test]
fn no_mistakes_config_json_matches_snapshot() {
    // The rule reports the loaded config. An outside --config path is absolute,
    // so only empty reports are snapshotted.
    let pass = rules_path(RULE, "fixture", "pass");
    assert_saved_fixture(
        "no_mistakes_config",
        &[
            (
                "pass",
                pass.clone(),
                rules_path(RULE, "configs", "pass.yml"),
            ),
            // `<<: *shared` is not expanded, so apps/missing is not a project root.
            (
                "merge_key",
                rules_path(RULE, "fixture", "merge-key"),
                rules_path(RULE, "configs", "merge-key.yml"),
            ),
            (
                "repo_root_dot",
                pass.clone(),
                rules_path(RULE, "configs", "repo-root-dot.yml"),
            ),
            (
                "env_all_limit",
                pass,
                rules_path(RULE, "configs", "env-all-limit.yml"),
            ),
        ],
    );
}
