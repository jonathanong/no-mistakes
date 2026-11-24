use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "github-actions-env-policy";

#[test]
fn github_actions_env_policy_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    assert_saved_fixture(
        "github_actions_env_policy",
        &[
            // 5 findings: workflow, job, step and composite scopes plus a forbidden name with an
            // expression value; two `${{ }}` required values are advisories, not mismatches.
            (
                "violations",
                rules_path(RULE, "fixture", "violations"),
                policy.clone(),
            ),
            // 0 findings: /dev/null, unconfigured names, and names inside run text.
            ("pass", rules_path(RULE, "fixture", "pass"), policy.clone()),
            // 0 findings: no options means nothing is required or forbidden.
            (
                "empty",
                rules_path(RULE, "fixture", "violations"),
                rules_path(RULE, "configs", "empty.yml"),
            ),
            // 2 findings: a quoted ` #` is not a comment, so the quoted key and value are located.
            (
                "quoted-hash",
                rules_path(RULE, "fixture", "quoted-hash"),
                rules_path(RULE, "configs", "quoted.yml"),
            ),
            // 0 findings, 2 advisories: non-mapping `env: ${{ fromJSON(...) }}` at job and step
            // scope is unanalyzable rather than silently skipped; empty `env:` stays quiet.
            (
                "dynamic-env",
                rules_path(RULE, "fixture", "dynamic-env"),
                policy.clone(),
            ),
            // 1 finding: line and next-line directives hide two of three.
            ("disabled", rules_path(RULE, "fixture", "disabled"), policy),
        ],
    );
}
