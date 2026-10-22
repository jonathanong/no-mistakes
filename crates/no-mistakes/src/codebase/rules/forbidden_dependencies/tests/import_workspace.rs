use super::*;

#[test]
fn import_workspace_relationships_exclude_resource_only_paths() {
    let root = fixture("forbidden-dependencies-import-workspace");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let findings = check(&root, &config, None).unwrap();
    let targets = findings
        .iter()
        .filter_map(|finding| finding.target.as_deref())
        .collect::<Vec<_>>();

    assert!(
        targets.contains(&"sharp"),
        "expected import finding: {findings:?}"
    );
    assert!(
        targets
            .iter()
            .any(|target| target.ends_with("packages/local/src/worker.mts")),
        "expected workspace-chain finding: {findings:?}"
    );
    // The fixture intentionally contains a resource edge so broadening this
    // import boundary back to the default relationship set fails loudly.
    assert!(
        !targets
            .iter()
            .any(|target| target.ends_with("resource-only.txt")),
        "resource-only target must stay outside the import boundary: {findings:?}"
    );
}
