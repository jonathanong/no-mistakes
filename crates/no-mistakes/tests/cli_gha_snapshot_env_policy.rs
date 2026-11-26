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
            // 0 findings, 5 advisories: non-mapping `jobs`, job, step and non-sequence `steps`
            // (including composite `runs.steps`) are unanalyzable rather than silently skipped.
            (
                "non-mapping-scopes",
                rules_path(RULE, "fixture", "non-mapping-scopes"),
                policy.clone(),
            ),
            // 1 finding: line and next-line directives hide two of three.
            ("disabled", rules_path(RULE, "fixture", "disabled"), policy),
        ],
    );
}

#[test]
fn github_actions_env_policy_invalid_yaml_json_matches_snapshot() {
    let root = rules_path(RULE, "fixture", "invalid");
    let cases = ["policy", "input-message", "ignore"].map(|config| {
        (
            config,
            root.clone(),
            rules_path(RULE, "configs", &format!("{config}.yml")),
        )
    });
    assert_saved_fixture("github_actions_env_policy_invalid_yaml", &cases);
}

#[cfg(unix)]
#[test]
fn github_actions_env_policy_unreadable_json_matches_snapshot() {
    let fixture =
        super::saved_fixture::materialize_path(&rules_path(RULE, "fixture", "unreadable"));
    let root = fixture.path().to_path_buf();
    let workflow = ".github/workflows/ci.yml";
    for args in [
        &["init", "--quiet"][..],
        &["add", "--force", "--", workflow],
    ] {
        let output = std::process::Command::new("git")
            .current_dir(&root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // A tracked workflow can become unreadable after discovery trusts its index mode.
    std::fs::rename(root.join("unavailable-workflow.link"), root.join(workflow)).unwrap();
    let cases = ["policy", "input-message", "ignore"].map(|config| {
        (
            config,
            root.clone(),
            rules_path(RULE, "configs", &format!("{config}.yml")),
        )
    });
    assert_saved_fixture("github_actions_env_policy_unreadable", &cases);
}
