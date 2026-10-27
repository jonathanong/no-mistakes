use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-job-timeouts";

#[test]
fn github_actions_job_timeouts_json_matches_snapshot() {
    assert_saved_fixture(
        "github_actions_job_timeouts",
        &[
            (
                "missing_timeout",
                rules_path(RULE, "fixture", "fail-missing"),
                rules_path(RULE, "configs", "max-minutes-10.yml"),
            ),
            (
                "over_cap",
                rules_path(RULE, "fixture", "fail-cap"),
                rules_path(RULE, "configs", "max-minutes-10.yml"),
            ),
            (
                "within_cap",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "max-minutes-10.yml"),
            ),
            (
                "reusable",
                rules_path(RULE, "fixture", "reusable"),
                rules_path(RULE, "configs", "max-minutes-10.yml"),
            ),
            (
                "invalid_yaml",
                rules_path(RULE, "fixture", "invalid"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "step_exceeds",
                rules_path(RULE, "fixture", "step-exceeds"),
                rules_path(RULE, "configs", "step-exceeds.yml"),
            ),
            (
                "allow_raises_cap",
                rules_path(RULE, "fixture", "allow"),
                rules_path(RULE, "configs", "allow-raises-cap.yml"),
            ),
            (
                "allow_without_max",
                rules_path(RULE, "fixture", "allow"),
                rules_path(RULE, "configs", "allow-without-max.yml"),
            ),
            (
                "allow_over_max",
                rules_path(RULE, "fixture", "allow"),
                rules_path(RULE, "configs", "allow-over-max.yml"),
            ),
            (
                "stale_allow",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "stale-allow.yml"),
            ),
            (
                "include_match",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "include-ci.yml"),
            ),
            (
                "include_miss",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "include-miss.yml"),
            ),
            (
                "no_cap",
                rules_path(RULE, "fixture", "pass"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "disable_file",
                rules_path(RULE, "fixture", "disable"),
                rules_path(RULE, "configs", "max-minutes-10.yml"),
            ),
            (
                "step_sum_off",
                rules_path(RULE, "fixture", "step-sum"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            (
                "step_sum_false",
                rules_path(RULE, "fixture", "step-sum"),
                rules_path(RULE, "configs", "reject-sum-false.yml"),
            ),
            (
                "step_sum",
                rules_path(RULE, "fixture", "step-sum"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "allow_sum",
                rules_path(RULE, "fixture", "step-sum"),
                rules_path(RULE, "configs", "allow-sum.yml"),
            ),
            (
                "allow_sum_max",
                rules_path(RULE, "fixture", "step-sum"),
                rules_path(RULE, "configs", "allow-sum-max.yml"),
            ),
            (
                "step_sum_disabled",
                rules_path(RULE, "fixture", "step-sum-disabled"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "step_sum_suppressed",
                rules_path(RULE, "fixture", "step-sum-suppressed"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "step_sum_and_exceeds",
                rules_path(RULE, "fixture", "step-sum"),
                rules_path(RULE, "configs", "both-rejects.yml"),
            ),
            (
                "exceeds_with_sum",
                rules_path(RULE, "fixture", "step-exceeds"),
                rules_path(RULE, "configs", "both-rejects.yml"),
            ),
            (
                "quoted_sum",
                rules_path(RULE, "fixture", "step-sum-quoted"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "quoted_cap",
                rules_path(RULE, "fixture", "step-sum-quoted"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "indented_sum",
                rules_path(RULE, "fixture", "step-sum-indented"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "inline_sum",
                rules_path(RULE, "fixture", "step-sum-inline"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "explicit_sum",
                rules_path(RULE, "fixture", "step-sum-explicit"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "explicit_scalar_sum",
                rules_path(RULE, "fixture", "step-sum-explicit-scalar"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "explicit_scalar_cap",
                rules_path(RULE, "fixture", "step-sum-explicit-scalar"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "alias_keys_sum",
                rules_path(RULE, "fixture", "step-sum-alias-keys"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "alias_keys_cap",
                rules_path(RULE, "fixture", "step-sum-alias-keys"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "flow_alias_sum",
                rules_path(RULE, "fixture", "step-sum-flow-alias"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "flow_alias_cap",
                rules_path(RULE, "fixture", "step-sum-flow-alias"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "root_alias_sum",
                rules_path(RULE, "fixture", "step-sum-root-alias"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "root_alias_cap",
                rules_path(RULE, "fixture", "step-sum-root-alias"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "map_alias_sum",
                rules_path(RULE, "fixture", "step-sum-map-alias"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "map_alias_cap",
                rules_path(RULE, "fixture", "step-sum-map-alias"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "multiline_sum",
                rules_path(RULE, "fixture", "step-sum-multiline"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "multiline_cap",
                rules_path(RULE, "fixture", "step-sum-multiline"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
            (
                "tags_sum",
                rules_path(RULE, "fixture", "step-sum-tags"),
                rules_path(RULE, "configs", "reject-sum.yml"),
            ),
            (
                "tags_cap",
                rules_path(RULE, "fixture", "step-sum-tags"),
                rules_path(RULE, "configs", "max-minutes-9.yml"),
            ),
        ],
    );
}
