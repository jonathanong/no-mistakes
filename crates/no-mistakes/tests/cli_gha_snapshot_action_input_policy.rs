use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-action-input-policy";

#[test]
fn github_actions_action_input_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    let invalid = rules_path("options-audit", "fixture", "issue-3958");
    assert_saved_fixture(
        "github_actions_action_input_policy",
        &[
            (
                "invalid_required_input",
                invalid.clone(),
                invalid.join("action-input.yml"),
            ),
            // 8 findings: wrong widget values, a pinned SHA, runtime bans and
            // requirements, and one banned ship input. Unlisted actions stay out.
            (
                "violations",
                rules_path(RULE, "fixture", "violations"),
                policy.clone(),
            ),
            // 0 findings: quoted and bare scalars match, and ban-only inputs may be absent.
            ("pass", rules_path(RULE, "fixture", "pass"), policy.clone()),
            // 0 findings: an empty actions list does not require or ban anything.
            (
                "empty_actions",
                rules_path(RULE, "fixture", "violations"),
                rules_path(RULE, "configs", "empty-actions.yml"),
            ),
            // 0 findings: include drops the workflow before inputs are judged.
            (
                "include_miss",
                rules_path(RULE, "fixture", "violations"),
                rules_path(RULE, "configs", "include-miss.yml"),
            ),
            // 0 findings: invalid YAML is skipped.
            (
                "invalid_yaml",
                rules_path(RULE, "fixture", "invalid"),
                policy.clone(),
            ),
            // 1 finding: the line directive hides keep-days on the first step only.
            (
                "disabled_line",
                rules_path(RULE, "fixture", "disabled-line"),
                policy.clone(),
            ),
            // 0 findings: a file directive hides every input in the workflow.
            (
                "disabled_file",
                rules_path(RULE, "fixture", "disabled-file"),
                policy.clone(),
            ),
            // 1 finding: a composite step keeps the wrong artifact lifetime.
            (
                "composite",
                rules_path(RULE, "fixture", "composite"),
                policy.clone(),
            ),
            // 2 findings: reusable calls miss keep-days, including a case variant.
            (
                "reusable",
                rules_path(RULE, "fixture", "reusable"),
                policy.clone(),
            ),
            // 3 findings: an unlisted cache input, a missing required-present
            // retention-days, and an unlisted overwrite input.
            (
                "allowlist_violations",
                rules_path(RULE, "fixture", "allowlist-violations"),
                rules_path(RULE, "configs", "allowlist.yml"),
            ),
            // 0 findings: allowed keys only, and a family that is not configured.
            (
                "allowlist_pass",
                rules_path(RULE, "fixture", "allowlist-pass"),
                rules_path(RULE, "configs", "allowlist.yml"),
            ),
            // Merge keys keep their meaning: << is an input named <<, so the
            // allowlist reports it and ignores the keys nested under it.
            (
                "allowlist_merge_keys",
                rules_path(RULE, "fixture", "merge-keys"),
                rules_path(RULE, "configs", "allowlist.yml"),
            ),
            // 6 findings: keys under << are not inputs. A sibling value and a
            // sequence still miss the nested requirements. An alias of with: passes.
            (
                "merge_keys",
                rules_path(RULE, "fixture", "merge-keys"),
                policy,
            ),
        ],
    );
}
