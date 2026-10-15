use super::*;

#[test]
fn returned_executors_factory_exports_and_client_parameters_keep_provenance() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        (8..=15).chain([26]).collect::<Vec<_>>(),
        "{findings:#?}"
    );
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "src/helpers.mts" && finding.line == 3),
        "{findings:#?}"
    );
}

#[test]
fn value_provenance_rejects_reassignments_conflicts_and_unrelated_receivers() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/controls.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 17, 24, 27, 39],
        "{findings:#?}"
    );
}

#[test]
fn returned_executor_selectors_mark_visible_factories_without_selecting_creation_calls() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("returned-only.yml")), None)
            .unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/entry.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [8, 9],
        "{findings:#?}"
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("transaction client only")));
}

#[test]
fn review_regressions_preserve_call_inputs_returns_mutations_and_factory_origins() {
    let root = root().with_file_name("provenance-fixture");
    let findings =
        crate::codebase::rules::run_check(&root, Some(&root.join("review.yml")), None).unwrap();
    let review = findings
        .iter()
        .filter(|finding| finding.file == "src/review.mts")
        .collect::<Vec<_>>();
    assert_eq!(
        review
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [9, 25, 26, 28],
        "{findings:#?}"
    );
    assert!(review
        .iter()
        .filter(|finding| finding.line != 9)
        .all(|finding| finding.message.contains("transaction client only")));
    let wrapper =
        crate::codebase::rules::run_check(&root, Some(&root.join("wrapper-only.yml")), None)
            .unwrap();
    assert_eq!(
        wrapper
            .iter()
            .filter(|finding| finding.file == "src/review.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [25],
        "{wrapper:#?}"
    );
    let marker =
        crate::codebase::rules::run_check(&root, Some(&root.join("review-marker.yml")), None)
            .unwrap();
    assert!(
        marker.iter().any(|finding| finding.file == "src/review.mts"
            && finding.line == 28
            && finding.message.contains("transaction client only")),
        "{marker:#?}"
    );
}

#[test]
fn mutation_aliases_completion_adapters_constructors_and_redeclarations_keep_provenance_sound() {
    let root = root().with_file_name("provenance-fixture");
    let findings = crate::codebase::rules::run_check(&root, None, None).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|finding| finding.file == "src/adapters-and-mutations.mts")
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [11, 15, 19, 23, 29],
        "{findings:#?}"
    );
}
