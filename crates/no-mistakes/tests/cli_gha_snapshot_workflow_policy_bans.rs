use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn check_json_snapshot_workflow_policy_bans_events() {
    let cases: Vec<_> = [
        ("banned", "forbidden-events.yml"),
        ("empty", "forbidden-events-empty.yml"),
        ("custom-message", "forbidden-events-custom.yml"),
        ("disabled", "forbidden-events-disabled.yml"),
        ("invalid-options", "forbidden-events-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path("workflow-topology-policy", "fixture", "forbidden-events"),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_policy_bans_events", &cases);
}

#[test]
fn check_json_snapshot_workflow_policy_bans_secrets() {
    let cases: Vec<_> = [
        ("banned", "secrets-inherit.yml"),
        ("disabled-false", "secrets-inherit-false.yml"),
        ("custom-message", "secrets-inherit-custom.yml"),
        ("disabled", "secrets-inherit-disabled.yml"),
        ("invalid-options", "secrets-inherit-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path("workflow-topology-policy", "fixture", "secrets-inherit"),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_policy_bans_secrets", &cases);
}

#[test]
fn check_json_snapshot_workflow_policy_bans_reusable_concurrency() {
    let cases: Vec<_> = [
        ("banned", "reusable-concurrency.yml"),
        ("disabled-false", "reusable-concurrency-false.yml"),
        ("custom-message", "reusable-concurrency-custom.yml"),
        ("disabled", "reusable-concurrency-disabled.yml"),
        ("invalid-options", "reusable-concurrency-invalid.yml"),
    ]
    .into_iter()
    .map(|(label, config)| {
        (
            label,
            rules_path(
                "workflow-topology-policy",
                "fixture",
                "reusable-concurrency",
            ),
            rules_path("workflow-topology-policy", "configs", config),
        )
    })
    .collect();
    assert_saved_fixture("workflow_policy_bans_reusable_concurrency", &cases);
}

#[test]
fn check_json_snapshot_ci_topology_malformed_workflow_diagnostic() {
    let root = rules_path(
        "workflow-topology-policy",
        "fixture",
        "reusable-concurrency",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .current_dir(&root)
        .args(["--lock-timeout", "0", "ci", "topology", "--root"])
        .arg(&root)
        .args(["--workflow", "malformed.yml", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    insta::with_settings!({
        snapshot_path => "common/snapshots",
        prepend_module_to_snapshot => false,
        omit_expression => true,
    }, {
        insta::assert_snapshot!(
            "ci_topology_malformed_workflow_diagnostic",
            String::from_utf8(output.stderr).unwrap().replace("\r\n", "\n")
        );
    });
}

#[test]
fn check_json_snapshot_workflow_policy_bans_reusable_concurrency_through_merges() {
    let cases = [(
        "merge-keys",
        rules_path("workflow-topology-policy", "fixture", "merge-keys"),
        rules_path(
            "workflow-topology-policy",
            "configs",
            "reusable-concurrency.yml",
        ),
    )];
    assert_saved_fixture("workflow_policy_merge_keys", &cases);
}

#[test]
fn check_json_snapshot_workflow_policy_valid_nested_merges() {
    const RULE: &str = "workflow-topology-policy";
    let cases = [(
        "valid-nested-merges",
        rules_path(RULE, "fixture", "valid-nested-merges"),
        rules_path(RULE, "configs", "valid-nested-merges.yml"),
    )];
    assert_saved_fixture("workflow_policy_valid_nested_merges", &cases);
}

#[test]
fn check_json_snapshot_workflow_policy_scopes_unlocated_merge() {
    const RULE: &str = "workflow-topology-policy";
    let mut cases = ["report", "advise", "ignore"]
        .map(|mode| {
            (
                mode,
                rules_path(RULE, "fixture", "merge-location-scoped"),
                rules_path(
                    RULE,
                    "configs",
                    &format!("merge-location-scoped-{mode}.yml"),
                ),
            )
        })
        .to_vec();
    cases.push((
        "inventory",
        rules_path(RULE, "fixture", "merge-location-inventory"),
        rules_path(RULE, "configs", "merge-location-scoped-inventory.yml"),
    ));
    assert_saved_fixture("workflow_policy_merge_location_scoped", &cases);
}
