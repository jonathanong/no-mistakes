use crate::codebase::ci_workflows::PreparedWorkflowTopologyInputs;
use crate::codebase::ts_source::VisiblePathSnapshot;
use std::path::Path;

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
