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

#[test]
fn generators_captured_mutations_and_parameter_initialization_are_conservative() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/review-contexts.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [19, 24, 28, 30],
        "{findings:#?}"
    );
}

#[test]
fn inherited_constructors_forward_inputs_and_preserve_defaults() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/inherited-constructors.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 12, 18],
        "{findings:#?}"
    );
}

#[test]
fn switch_tags_void_defaults_and_nested_mutations_preserve_value_flow() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/syntax-and-nesting.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7, 12, 18, 22],
        "{findings:#?}"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/default-executor-entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [3, 4],
        "{findings:#?}"
    );
}

#[test]
fn destructured_parameters_project_before_caller_agreement() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/destructured-parameters.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 6, 8, 17, 21, 31, 43, 45, 52],
        "{findings:#?}"
    );
}

#[test]
fn selected_default_origins_track_receiver_mutations_and_copied_methods() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/default-origin-mutations.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [10, 13, 16, 19, 25, 28, 32],
        "{findings:#?}"
    );
}

#[test]
fn default_mutation_paths_late_declarations_and_inherited_fields_are_conservative() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    for (file, expected) in [
        ("src/default-review-stars.mts", vec![6]),
        ("src/default-review-late.mts", vec![12]),
        ("src/default-review-inherited.mts", vec![7, 9, 14]),
    ] {
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.file == file)
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            expected,
            "{findings:#?}"
        );
    }
}
