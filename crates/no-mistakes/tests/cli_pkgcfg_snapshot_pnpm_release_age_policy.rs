use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "pnpm-release-age-policy";

fn case(
    label: &'static str,
    scenario: &'static str,
    config: &'static str,
) -> (&'static str, PathBuf, PathBuf) {
    (
        label,
        rules_path(RULE, "fixture", scenario),
        rules_path(RULE, "configs", config),
    )
}

fn saved_case(
    label: &'static str,
    root: &str,
    name: &'static str,
    config: &'static str,
) -> (&'static str, PathBuf, PathBuf) {
    (
        label,
        no_mistakes::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/rules")
                .join(root)
                .join(name),
        ),
        rules_path(RULE, "configs", config),
    )
}

#[test]
fn pnpm_release_age_policy_json_matches_snapshot() {
    let cases = vec![
        case("pass", "pass", "registered.yml"),
        case("fail_exclude", "fail-exclude", "registered.yml"),
        case("fail_dependabot", "fail-dependabot", "registered.yml"),
        case("fail_lockfile", "fail-lockfile", "registered.yml"),
        case("fail_graph", "fail-graph", "registered.yml"),
        case("empty_options", "fail-exclude", "empty.yml"),
        case(
            "malformed_workspace",
            "malformed-workspace",
            "registered.yml",
        ),
        case(
            "invalid_workspace_yaml",
            "invalid-workspace-yaml",
            "registered.yml",
        ),
        case("non_mapping", "non-mapping-workspace", "registered.yml"),
        case("non_mapping_scan", "non-mapping-scan", "registered.yml"),
        case("grouped_pass", "pass", "grouped.yml"),
        case("invalid_groups", "pass", "invalid-groups.yml"),
        case("duplicate_selectors", "pass", "duplicate-selectors.yml"),
        case("past_eligibility", "pass", "past-eligibility.yml"),
        case("included_exclude", "included-exclude", "registered.yml"),
        case("duplicate_exclude", "duplicate-exclude", "registered.yml"),
        case("unreadable_exclude", "unreadable-exclude", "registered.yml"),
        // The unit file list always named dependabot.yml, even when absent.
        saved_case(
            "pnpm12_project",
            "pnpm-release-age-policy",
            "pnpm12-project-package",
            "registered-dependabot-path.yml",
        ),
        saved_case(
            "pnpm12_env",
            "pnpm-release-age-policy",
            "pnpm12-env-package",
            "registered-dependabot-path.yml",
        ),
        saved_case(
            "pnpm12_absent",
            "pnpm-release-age-policy",
            "pnpm12-absent-project",
            "registered-dependabot-path.yml",
        ),
        saved_case(
            "issue_two_doc",
            "pnpm12-issue-1035",
            "two-doc",
            "issue-1035.yml",
        ),
        saved_case(
            "issue_single_doc",
            "pnpm12-issue-1035",
            "single-doc",
            "issue-1035.yml",
        ),
        saved_case(
            "issue_malformed",
            "pnpm12-issue-1035",
            "malformed",
            "issue-1035.yml",
        ),
        saved_case(
            "issue_non_env",
            "pnpm12-issue-1035",
            "non-env",
            "issue-1035.yml",
        ),
        saved_case(
            "issue_missing_lock",
            "pnpm12-issue-1035",
            "two-doc",
            "issue-missing-lock.yml",
        ),
    ];
    assert_saved_fixture("pnpm_release_age_policy", &cases);
}
