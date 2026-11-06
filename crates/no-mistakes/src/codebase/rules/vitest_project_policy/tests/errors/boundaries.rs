use super::*;

fn boundary(case: &str) -> Diagnostics {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-project-policy/fixture/lexical-boundaries"),
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
fn cyclic_unresolved_and_nonliteral_property_values_do_not_become_missing_properties() {
    for case in [
        "local-cycle",
        "unary-string",
        "unary-operator",
        "nonfinite-number",
        "unary-nonfinite",
        "missing-member",
        "missing-namespace-member",
        "unknown-expression",
        "unresolved-local-export",
        "ambiguous-star-exports",
        "opaque-star-exports",
    ] {
        let result = boundary(case);
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
}

#[test]
fn opaque_project_entries_and_invalid_vitest_wrapper_arguments_are_incomplete() {
    for case in [
        "opaque-array-spread",
        "wrapper-no-arguments",
        "wrapper-spread-argument",
        "wrapper-multiple-arguments",
    ] {
        let result = boundary(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert!(!result.incomplete.is_empty(), "{case}");
    }
}

#[test]
fn computed_and_method_properties_do_not_prove_absence_of_required_or_forbidden_keys() {
    for case in ["computed-property", "method-property"] {
        let result = boundary(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        let properties = result
            .incomplete
            .iter()
            .map(|failure| failure.property.as_deref())
            .collect::<Vec<_>>();
        assert!(
            properties.contains(&Some("isolate")),
            "{case}: {properties:?}"
        );
        assert!(properties.contains(&Some("pool")), "{case}: {properties:?}");
    }
}

#[test]
fn repeated_helpers_and_erased_type_exports_preserve_known_runtime_properties() {
    for case in [
        "repeated-helper",
        "known-namespace-member",
        "only-type-reexports",
    ] {
        let result = boundary(case);
        assert!(result.findings.is_empty(), "{case}: {:?}", result.findings);
        assert!(
            result.incomplete.is_empty(),
            "{case}: {:?}",
            result.incomplete
        );
    }
}

#[test]
fn unsupported_export_declarations_preserve_known_literals_without_claiming_unknown_absence() {
    let result = boundary("unsupported-export-declarations");
    assert!(result.findings.is_empty(), "{:?}", result.findings);
    assert_eq!(result.incomplete.len(), 1, "{:?}", result.incomplete);
    assert_eq!(result.incomplete[0].property.as_deref(), Some("pool"));
}
