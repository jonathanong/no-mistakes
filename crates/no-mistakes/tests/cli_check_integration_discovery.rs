use std::path::PathBuf;

#[test]
fn check_json_snapshot_integration_discovery_scope() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/integration-tests");
    let root = base.join("discovery-scope/fixture");
    let mut cases = [
        ("scoped", ".no-mistakes.yml"),
        ("mixed_rules", "mixed.yml"),
        ("all_files", "all-files.yml"),
        ("disabled", "disabled.yml"),
        ("empty_scope", "empty-scope.yml"),
        ("invalid_include", "invalid-include.yml"),
        ("invalid_exclude", "invalid-exclude.yml"),
        ("runner_derived", "runner-derived.yml"),
    ]
    .into_iter()
    .map(|(name, config)| (name, root.clone(), root.join(config)))
    .collect::<Vec<_>>();
    for (name, directory) in [
        ("parse_error", "discovery-scope-errors"),
        ("namespace_error", "discovery-scope-namespace-error"),
    ] {
        let root = base.join(directory).join("fixture");
        cases.push((name, root.clone(), root.join(".no-mistakes.yml")));
    }
    crate::check_json_snapshot::assert_saved_fixture("integration_discovery_scope", &cases);
}

#[test]
fn check_json_snapshot_structural_sql_scope() {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/check-facts");
    let root = base.join("sql-support-scope/fixture");
    let mut cases = [
        ("scoped", ".no-mistakes.yml"),
        ("broad", "broad.yml"),
        ("low_fallback", "low.yml"),
        ("middle_exclusion", "middle-exclude.yml"),
    ]
    .into_iter()
    .map(|(name, config)| (name, root.clone(), root.join(config)))
    .collect::<Vec<_>>();
    let audit = base.join("sql-audit-scope/fixture");
    cases.push((
        "active_audit",
        audit.clone(),
        audit.join(".no-mistakes.yml"),
    ));
    let root = base.join("sql-no-support/fixture");
    cases.push(("no_support", root.clone(), root.join(".no-mistakes.yml")));
    cases.push(("all_selected", root.clone(), root.join("all-selected.yml")));
    cases.push(("empty_scope", root.clone(), root.join("empty.yml")));
    crate::check_json_snapshot::assert_saved_fixture("structural_sql_scope", &cases);
}
