use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

const RULE: &str = "tsconfig-gate-coverage";

fn gate_root(name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/tsconfig-gate-coverage")
            .join(name),
    )
}

fn config(name: &str) -> PathBuf {
    rules_path(RULE, "configs", name)
}

fn in_fixture(name: &str) -> (PathBuf, PathBuf) {
    let root = gate_root(name);
    let config = root.join(".no-mistakes.yml");
    (root, config)
}

#[test]
fn tsconfig_gate_coverage_json_matches_snapshot() {
    // Former check() output. Allowlist and fixture-policy configs live inside
    // the fixture so finding files stay repository-relative. Workflow expression
    // helpers that never call check stay in the unit tests.
    let mut cases = vec![
        (
            "missing_ci",
            gate_root("missing-ci"),
            config("missing-ci.yml"),
        ),
        (
            "missing_local",
            gate_root("missing-local"),
            config("missing-local.yml"),
        ),
        (
            "working_directories",
            gate_root("working-directories"),
            config("working-directories.yml"),
        ),
        (
            "non_enforcing",
            gate_root("non-enforcing-workflow"),
            config("non-enforcing.yml"),
        ),
        (
            "allowlist_errors",
            gate_root("allowlist-errors"),
            // The finding file is the loaded config. It has to live inside
            // the fixture so the path stays `.no-mistakes.yml` on every OS.
            gate_root("allowlist-errors").join(".no-mistakes.yml"),
        ),
        (
            "allowlist_pass",
            gate_root("allowlist-pass"),
            config("allowlist-pass.yml"),
        ),
        (
            "malformed_workflow",
            gate_root("malformed-workflow"),
            config("malformed-workflow.yml"),
        ),
        (
            "include_app",
            gate_root("missing-ci"),
            config("include-app.yml"),
        ),
    ];
    // Saved-fixture reports from fixture_policy.rs. Absence of a project from
    // the rules array is the old negative assertion.
    for (label, name) in [
        ("directory_pass", "pass"),
        ("path_filter_negative", "path-filter-sources-negative"),
        ("path_filter_positive", "path-filter-sources-positive"),
        ("reusable_workflow", "reusable-workflow"),
        ("local_action_targets", "local-action-targets"),
        ("activity_conditions", "activity-conditions"),
        ("reusable_activation", "reusable-activation-state"),
        ("reusable_review", "reusable-review-regressions"),
        ("branch_intersection", "reusable-branch-intersection"),
        ("missing_working_directory", "nonexistent-working-directory"),
        ("reusable_static_outputs", "reusable-static-outputs"),
        ("tolerated_failure_needs", "tolerated-failure-needs"),
        ("unsupported_case", "unsupported-case-expression"),
        ("no_check", "no-check"),
    ] {
        let (root, config) = in_fixture(name);
        cases.push((label, root, config));
    }
    assert_saved_fixture("tsconfig_gate_coverage", &cases);
}
