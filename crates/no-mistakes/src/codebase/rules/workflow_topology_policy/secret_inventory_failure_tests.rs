use super::*;
use crate::codebase::ci_workflows::ParsedWorkflowSet;
use std::sync::Arc;

// Output cases moved to cli_gha_snapshot_workflow_topology.

#[test]
fn unknown_config_path_is_explicitly_incomplete() {
    let root = fixture("unused");
    let path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&path)).unwrap();
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let inputs =
        PreparedWorkflowTopologyInputs::for_config(&root, &config, &snapshot, &sources).unwrap();
    let findings = check_prepared(
        &root,
        &config,
        &snapshot.paths_for(&root),
        &sources,
        &inputs,
        None,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, ".");
    assert_eq!(findings[0].line, 1);
    assert!(findings[0]
        .message
        .contains("configuration path unavailable for unused declarations"));
}

#[test]
fn unavailable_config_source_is_cached_as_incomplete_across_repeated_checks() {
    let saved = crate::test_support::materialize_saved_fixture(&fixture("unused"));
    let root = saved.path();
    let path = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(root, Some(&path)).unwrap();
    let snapshot = VisiblePathSnapshot::new(root);
    let sources = snapshot.source_store_for(root);
    let inputs =
        PreparedWorkflowTopologyInputs::for_config(root, &config, &snapshot, &sources).unwrap();
    // Remove only the per-test copy after loading its config, before preparing config locations.
    std::fs::remove_file(&path).unwrap();
    let first = check_prepared(
        root,
        &config,
        &snapshot.paths_for(root),
        &sources,
        &inputs,
        Some(&path),
    )
    .unwrap();
    let reads = sources.physical_read_count();
    let second = check_prepared(
        root,
        &config,
        &snapshot.paths_for(root),
        &sources,
        &inputs,
        Some(&path),
    )
    .unwrap();
    assert_eq!(first, second);
    assert_eq!(sources.physical_read_count(), reads);
    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(first[0].file, ".no-mistakes.yml");
    assert_eq!(first[0].line, 1);
    assert!(first[0]
        .message
        .contains("configuration source unavailable for unused declarations"));
}

#[test]
fn retained_workflow_and_action_read_failures_prevent_unused_and_do_not_retry_io() {
    for (case, removed, action) in [
        ("unused", ".github/workflows/check.yml", false),
        ("composite", "actions/example/action.yml", true),
    ] {
        let saved = crate::test_support::materialize_saved_fixture(&fixture(case));
        let root = saved.path();
        let path = root.join(".no-mistakes.yml");
        let config = crate::config::v2::load_v2_config(root, Some(&path)).unwrap();
        let snapshot = VisiblePathSnapshot::new(root);
        let sources = snapshot.source_store_for(root);
        // The requested saved path remains in the request inventory after its copied file disappears.
        std::fs::remove_file(root.join(removed)).unwrap();
        let workflows = ParsedWorkflowSet::from_paths_and_sources(
            root,
            [root.join(".github/workflows/check.yml")],
            &sources,
        );
        let actions = ParsedWorkflowSet::from_paths_and_sources(
            root,
            action.then(|| root.join(removed)),
            &sources,
        );
        let inputs =
            PreparedWorkflowTopologyInputs::new(root, &config.ci, Arc::new(workflows), actions);
        let first = check_prepared(
            root,
            &config,
            &snapshot.paths_for(root),
            &sources,
            &inputs,
            Some(&path),
        )
        .unwrap();
        let reads = sources.physical_read_count();
        let parses = sources.yaml_parse_count();
        let second = check_prepared(
            root,
            &config,
            &snapshot.paths_for(root),
            &sources,
            &inputs,
            Some(&path),
        )
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(sources.physical_read_count(), reads);
        assert_eq!(sources.yaml_parse_count(), parses);
        assert_eq!(first.len(), 1, "{case}: {first:?}");
        assert_eq!(first[0].file, removed);
        assert_eq!(first[0].line, 1);
        assert!(first[0].message.contains("document unavailable:"));
    }
}
