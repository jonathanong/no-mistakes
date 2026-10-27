use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-docker-host-ports";

#[test]
fn github_actions_docker_host_ports_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_docker_host_ports",
        &[
            (
                "pass",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "pass.yml"),
            ),
            (
                "abstain",
                rules_path(RULE, "fixture", "abstain"),
                rules_path(RULE, "configs", "abstain.yml"),
            ),
            (
                "fail",
                rules_path(RULE, "fixture", "fail"),
                rules_path(RULE, "configs", "fail.yml"),
            ),
            (
                "flow",
                rules_path(RULE, "fixture", "flow"),
                rules_path(RULE, "configs", "flow.yml"),
            ),
        ],
    );
}
