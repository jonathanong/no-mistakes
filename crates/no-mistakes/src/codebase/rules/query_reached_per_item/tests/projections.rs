use super::*;

#[test]
fn copied_members_defaults_and_undefined_apply_preserve_provenance() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/projections.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [5, 10, 11, 14, 16, 28],
        "{findings:#?}"
    );
}

#[test]
fn call_valued_export_selectors_match_local_and_imported_occurrences() {
    let root = root().with_file_name("provenance-fixture");
    let config = root.join("value-selectors.yml");
    let findings = crate::codebase::rules::run_check(&root, Some(&config), None).unwrap();
    assert_eq!(
        findings
            .iter()
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [("src/local-value-entry.mts", 3), ("src/local-value.mts", 3)],
        "{findings:#?}"
    );
    let report = crate::effects_query::run(
        &root,
        Some(&config),
        None,
        "postgres",
        Path::new("src/local-value-entry.mts"),
        &[],
        None,
    )
    .unwrap();
    assert_eq!(
        report
            .call_sites
            .iter()
            .map(|site| (site.file.as_str(), site.line))
            .collect::<Vec<_>>(),
        [("src/local-value-entry.mts", 3), ("src/local-value.mts", 3)],
        "{report:#?}"
    );
}

#[test]
fn async_factory_awaits_and_bound_arguments_preserve_provenance() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/async-and-bind.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 13, 19, 22, 29],
        "{findings:#?}"
    );
}

#[test]
fn imported_promises_and_bound_functions_keep_their_value_kinds() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("async-imports.yml")), None)
            .unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/async-binding-entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 4],
        "{findings:#?}"
    );
}
