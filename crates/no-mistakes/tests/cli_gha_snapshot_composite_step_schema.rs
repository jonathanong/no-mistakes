use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-composite-step-schema";

#[test]
fn github_actions_composite_step_schema_json_matches_snapshot() {
    let default = rules_path(RULE, "configs", "default.yml");
    assert_saved_fixture(
        "github_actions_composite_step_schema",
        &[
            (
                "timeout_minutes",
                rules_path(RULE, "fixture", "fail"),
                default.clone(),
            ),
            ("pass", rules_path(RULE, "fixture", "pass"), default.clone()),
            (
                "description",
                rules_path(RULE, "fixture", "description"),
                default.clone(),
            ),
            (
                "block_scalar",
                rules_path(RULE, "fixture", "block-scalar"),
                default.clone(),
            ),
            // The mapping key is line 9; the block-scalar mention is not a finding.
            (
                "block_scalar_and_key",
                rules_path(RULE, "fixture", "block-scalar-and-key"),
                default.clone(),
            ),
            (
                "disable_next_line",
                rules_path(RULE, "fixture", "disable-next-line"),
                default.clone(),
            ),
            (
                "disable_line",
                rules_path(RULE, "fixture", "disable-line"),
                default.clone(),
            ),
            (
                "docker",
                rules_path(RULE, "fixture", "docker"),
                default.clone(),
            ),
            (
                "invalid_yaml",
                rules_path(RULE, "fixture", "invalid"),
                default.clone(),
            ),
            (
                "default_include_skips_workflow",
                rules_path(RULE, "fixture", "cases/default-include-skips-workflow"),
                default.clone(),
            ),
            // Root action.yaml is outside the default `.github/actions/**` include.
            (
                "include_root_action_yaml",
                rules_path(RULE, "fixture", "cases/include-override-root-action"),
                rules_path(RULE, "configs", "include-action-yaml.yml"),
            ),
            (
                "extra_forbidden_env",
                rules_path(RULE, "fixture", "cases/extra-forbidden-env"),
                rules_path(RULE, "configs", "extra-forbidden-env.yml"),
            ),
            (
                "allowed_keys_keep_timeout",
                rules_path(RULE, "fixture", "cases/allowed-keys-keeps-timeout"),
                rules_path(RULE, "configs", "allowed-keys-keep-timeout.yml"),
            ),
            (
                "allowed_keys_drop_uses",
                rules_path(RULE, "fixture", "cases/allowed-keys-drops-uses"),
                rules_path(RULE, "configs", "allowed-keys-name-only.yml"),
            ),
            // Two findings: the step label falls back to uses, then to "step #2".
            (
                "label_fallback",
                rules_path(RULE, "fixture", "cases/label-fallback"),
                default.clone(),
            ),
            (
                "empty_name_fallback",
                rules_path(RULE, "fixture", "cases/empty-name-fallback"),
                default.clone(),
            ),
            (
                "disable_file",
                rules_path(RULE, "fixture", "cases/disable-file"),
                default.clone(),
            ),
            (
                "disable_file_invalid",
                rules_path(RULE, "fixture", "cases/disable-file-invalid"),
                default.clone(),
            ),
            (
                "missing_or_non_array_steps",
                rules_path(RULE, "fixture", "cases/missing-or-non-array-steps"),
                default.clone(),
            ),
            (
                "scalar_and_missing_runs",
                rules_path(RULE, "fixture", "cases/scalar-and-missing-runs"),
                default.clone(),
            ),
            (
                "null_and_non_mapping_steps",
                rules_path(RULE, "fixture", "cases/null-and-non-mapping-steps"),
                default.clone(),
            ),
            (
                "numeric_key",
                rules_path(RULE, "fixture", "cases/numeric-key"),
                default.clone(),
            ),
            // Only the quoted `using: "composite"` file is a finding.
            (
                "quoted_using",
                rules_path(RULE, "fixture", "cases/using-must-be-composite"),
                default.clone(),
            ),
            (
                "flow_style_timeout",
                rules_path(RULE, "fixture", "cases/flow-style-timeout"),
                default,
            ),
        ],
    );
}
