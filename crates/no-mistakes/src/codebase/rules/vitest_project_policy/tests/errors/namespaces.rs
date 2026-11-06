use super::*;

fn namespace(case: &str) -> Diagnostics {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture/options-errors"),
    );
    let files = vec![root.join(format!("{case}.ts"))];
    let sources = SourceStore::new(std::sync::Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    ));
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(format!("{case}.yml")))).unwrap();
    diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap()
}

#[test]
fn unavailable_namespace_exports_leave_required_and_forbidden_properties_incomplete() {
    for case in [
        "namespace-undeclared",
        "namespace-missing",
        "namespace-malformed",
        "namespace-mutable",
        "namespace-destructured",
    ] {
        let result = namespace(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert!(
            result
                .incomplete
                .iter()
                .any(|failure| failure.property.as_deref() == Some("isolate")),
            "{case}: {:?}",
            result.incomplete
        );
    }
    for case in [
        "namespace-undeclared",
        "namespace-missing",
        "namespace-malformed",
    ] {
        assert!(
            namespace(case)
                .incomplete
                .iter()
                .any(|failure| failure.property.as_deref() == Some("pool")),
            "{case}"
        );
    }
}

#[test]
fn partial_namespace_reexports_preserve_known_values_and_unknown_absence() {
    for case in ["namespace-partial", "namespace-star-unknown"] {
        let result = namespace(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert_eq!(
            result.incomplete.len(),
            1,
            "{case}: {:?}",
            result.incomplete
        );
        assert_eq!(result.incomplete[0].property.as_deref(), Some("pool"));
    }
}

#[test]
fn fully_known_namespaces_and_star_reexports_preserve_literal_properties() {
    for case in [
        "namespace-known",
        "namespace-star-known",
        "namespace-named-star",
    ] {
        let result = namespace(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert!(
            result.incomplete.is_empty(),
            "{case}: {:?}",
            result.incomplete
        );
    }
    let empty = namespace("namespace-empty");
    assert!(empty.incomplete.is_empty());
    assert_eq!(empty.findings.len(), 1);
    assert!(empty.findings[0]
        .message
        .contains("test.isolate must equal true"));
}
