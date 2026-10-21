use super::*;

#[test]
fn production_import_projection_skips_full_fact_finalization() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/codebase-analysis/import-facts/fixture/production-import-projection.mts",
    );
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed = oxc_parser::Parser::new(
        &allocator,
        &source,
        oxc_span::SourceType::from_path(&path).unwrap(),
    )
    .parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);

    FULL_PROJECTION_FINALIZATIONS.with(|count| count.set(0));
    RESOURCE_ROOT_CONSTRUCTIONS.with(|count| count.set(0));
    graph_call_recording_probe::reset();
    let imports = extract_imports_from_program_with_source(&parsed.program, &source);
    assert_eq!(FULL_PROJECTION_FINALIZATIONS.with(|count| count.get()), 0);
    assert_eq!(RESOURCE_ROOT_CONSTRUCTIONS.with(|count| count.get()), 0);
    assert_eq!(graph_call_recording_probe::counts(), (0, 0));
    let full = extract_import_facts_from_program_with_source(&parsed.program, &source);
    assert_eq!(FULL_PROJECTION_FINALIZATIONS.with(|count| count.get()), 1);
    assert!(RESOURCE_ROOT_CONSTRUCTIONS.with(|count| count.get()) > 0);
    let (expression_calls, unknown_calls) = graph_call_recording_probe::counts();
    assert!(expression_calls > 0);
    assert!(unknown_calls > 0);
    // Equality covers every ExtractedImport field, including scope and reachability.
    assert_eq!(imports, full.imports);
    assert!(imports.iter().any(|item| item.runtime_reachable));
    assert!(imports
        .iter()
        .any(|item| item.kind == ImportKind::RequireResolve));
    for specifier in [
        "./constructor-arg.js",
        "./tag-arg.js",
        "./optional-argument.js",
        "./unknown-callee-arg.js",
        "./scoped-unknown-callee-arg.js",
    ] {
        assert!(imports.iter().any(|item| item.specifier == specifier));
    }
    assert!(!imports.iter().any(|item| item.specifier == "./shadowed.js"));
}
