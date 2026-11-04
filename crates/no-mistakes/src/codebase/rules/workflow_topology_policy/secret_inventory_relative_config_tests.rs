use crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs;
use crate::codebase::ts_source::VisiblePathSnapshot;
use std::path::Path;

// Kept: `check --config` is absolute, so this relative path is not a snapshot case.

#[test]
fn unused_declarations_resolve_a_relative_config_path_against_root() {
    let root = super::fixture("unused");
    let absolute_config = root.join(".no-mistakes.yml");
    let config = crate::config::v2::load_v2_config(&root, Some(&absolute_config)).unwrap();
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let inputs =
        PreparedWorkflowTopologyInputs::for_config(&root, &config, &snapshot, &sources).unwrap();

    let findings = super::super::check_prepared(
        &root,
        &config,
        &snapshot.paths_for(&root),
        &sources,
        &inputs,
        Some(Path::new(".no-mistakes.yml")),
    )
    .unwrap();

    let unused: Vec<_> = findings
        .iter()
        .filter(|finding| finding.message.starts_with("unused declared secret"))
        .map(|finding| (finding.file.as_str(), finding.line))
        .collect();
    assert_eq!(unused, [(".no-mistakes.yml", 9), (".no-mistakes.yml", 12)]);
}

#[test]
fn split_inventory_reports_unused_secrets_on_the_authored_fragment() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/config-v2/split-provenance/fixture/rule-diagnostics"),
    );
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let inputs =
        PreparedWorkflowTopologyInputs::for_config(&root, &config, &snapshot, &sources).unwrap();
    let findings = super::super::check_prepared(
        &root,
        &config,
        &snapshot.paths_for(&root),
        &sources,
        &inputs,
        config.loaded_config_path.0.as_deref(),
    )
    .unwrap();
    let finding = findings
        .iter()
        .find(|finding| finding.message.starts_with("unused declared secret"))
        .unwrap();
    assert_eq!(finding.file, ".no-mistakes/rules/workflows.yml");
    assert_eq!(finding.line, 5);
}
