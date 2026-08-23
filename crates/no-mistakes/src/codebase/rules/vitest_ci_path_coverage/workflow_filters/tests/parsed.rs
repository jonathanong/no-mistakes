use super::super::{ci_filters_from_parsed_with_sources, CiFilter, WorkflowSelector};
use crate::codebase::ci_workflows::ParsedWorkflowSet;
use crate::codebase::rules::RuleFinding;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/vitest-ci-path-coverage/parsed-workflow-errors"),
    )
}

fn parsed_filters(
    root: &Path,
    paths: &[PathBuf],
    selectors: &[WorkflowSelector],
) -> (Vec<CiFilter>, Vec<RuleFinding>) {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, paths);
    let sources = snapshot.source_store_for(root);
    let parsed = ParsedWorkflowSet::from_paths(root, paths.iter().cloned());
    ci_filters_from_parsed_with_sources(root, selectors, &parsed, &sources)
}

#[test]
fn errors_keep_read_and_parse_context() {
    let root = fixture_root();
    let paths = [
        root.join(".github/workflows/bad.yml"),
        root.join(".github/workflows/missing.yml"),
    ];
    let (filters, findings) = parsed_filters(&root, &paths, &[]);
    assert!(filters.is_empty());
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("could not parse workflow YAML")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("could not read workflow file")));
}

#[test]
fn selectors_skip_nonmatching_documents_before_loading_errors() {
    let root = fixture_root();
    let paths = [root.join(".github/workflows/bad.yml")];
    let (filters, findings) = parsed_filters(
        &root,
        &paths,
        &[WorkflowSelector {
            path: ".github/workflows/selected.yml".to_string(),
            ..Default::default()
        }],
    );
    assert!(filters.is_empty());
    assert!(findings.is_empty());
}
