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
        [4, 6, 8, 17, 21, 31, 43, 45, 52, 57],
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
        ("src/default-review-inherited.mts", vec![7, 9, 14, 20]),
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

#[test]
fn imported_receivers_track_late_objects_and_supplied_arguments() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/imported-receiver-mutations.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [7, 21, 23, 25, 33],
        "{findings:#?}"
    );
}

#[test]
fn nested_alias_writes_do_not_report_stale_imported_effects() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/nested-receiver-writes.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 16, 18, 20, 41, 55, 73, 81],
        "{findings:#?}"
    );
}

#[test]
fn conditional_and_unknown_reference_writes_do_not_sever_original_receivers() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/nested-receiver-cutoffs.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [93, 175, 184, 207, 217, 268, 275],
        "{findings:#?}"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/nested-receiver-owned-defaults.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [12, 30],
        "{findings:#?}"
    );
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file.starts_with("src/nested-receiver-containers/"))
            .map(|finding| (finding.file.as_str(), finding.line))
            .collect::<Vec<_>>(),
        [
            ("src/nested-receiver-containers/copied-method.mts", 9),
            ("src/nested-receiver-containers/replaced-terminal.mts", 8),
            ("src/nested-receiver-containers/sibling.mts", 8),
        ],
        "{findings:#?}"
    );
}
