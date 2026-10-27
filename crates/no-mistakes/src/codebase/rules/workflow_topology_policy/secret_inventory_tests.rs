use super::*;
use crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs;
use crate::codebase::ts_source::VisiblePathSnapshot;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root(RULE_ID)
            .join("secret-inventory")
            .join(name),
    )
}

fn run(name: &str, file: &str) -> Result<Vec<RuleFinding>> {
    let root = fixture(name);
    let path = root.join(file);
    let config = crate::config::v2::load_v2_config(&root, Some(&path))?;
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let inputs =
        PreparedWorkflowTopologyInputs::for_config(&root, &config, &snapshot, &sources).unwrap();
    check_prepared(
        &root,
        &config,
        &snapshot.paths_for(&root),
        &sources,
        &inputs,
        Some(&path),
    )
}

// Output cases moved to cli_gha_snapshot_workflow_topology.

#[test]
fn compatibility_preparation_matches_the_request_scoped_action_scan() {
    let enabled = fixture("unreferenced-action");
    let enabled_config =
        crate::config::v2::load_v2_config(&enabled, Some(&enabled.join(".no-mistakes.yml")))
            .unwrap();
    assert_eq!(
        super::prepare::check_with_files(&enabled, &enabled_config, &[]).unwrap(),
        run("unreferenced-action", ".no-mistakes.yml").unwrap()
    );
}

#[test]
fn inventory_rejects_case_insensitive_duplicate_names() {
    let error = run("repository", "duplicate.yml").unwrap_err().to_string();
    assert!(error.contains("duplicate secretInventory name"), "{error}");
}

#[path = "secret_inventory_validation_tests.rs"]
mod validation;

#[path = "secret_inventory_failure_tests.rs"]
mod failure;

#[path = "secret_inventory_relative_config_tests.rs"]
mod relative_config;
