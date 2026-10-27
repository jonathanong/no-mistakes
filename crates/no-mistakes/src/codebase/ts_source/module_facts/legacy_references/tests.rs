use crate::codebase::ts_source::module_facts::{
    analyze_typescript_modules, TypeScriptModulesOptions,
};
use std::path::PathBuf;

#[test]
fn arguments_reference_metadata_matches_the_saved_0153_projection() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/typescript-module-facts/arguments-reference-compat");
    // The baseline includes every field, not just the restored references. The
    // invocation-object semantics are separately covered by helper tracing.
    let expected: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("baseline.json")).unwrap()).unwrap();
    for name in ["scopes.cjs", "no-binding.cjs", "types.ts", "ordinary.cjs"] {
        let report = analyze_typescript_modules(&TypeScriptModulesOptions {
            root: Some(root.clone()),
            files: vec![PathBuf::from(name)],
        })
        .unwrap();
        assert_eq!(
            serde_json::to_value(&report.modules[0].facts).unwrap(),
            expected[name],
            "{name}"
        );
    }
}

#[test]
fn sloppy_function_name_retains_legacy_reference_metadata() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-alternative-callback",
    );
    let report = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(root),
        files: vec![PathBuf::from("src/self-arguments.cjs")],
    })
    .unwrap();
    let binding = report.modules[0]
        .facts
        .bindings
        .iter()
        .find(|binding| binding.name == "arguments")
        .unwrap();
    assert_eq!((binding.span.start, binding.span.end), (110, 119));
    assert_eq!(binding.references.len(), 1);
    let reference = &binding.references[0];
    assert_eq!((reference.span.start, reference.span.end), (131, 140));
    assert!(reference.runtime);
    assert!(!reference.type_only);
}
