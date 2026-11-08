use super::*;
use std::path::PathBuf;

fn module(case: &str, file: &str) -> TypeScriptModule {
    analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../test-cases/rules/production-exports-only-used-by-tests/fixture/precision/{case}"
        ))),
        files: vec![PathBuf::from(file)],
    })
    .unwrap()
    .modules
    .remove(0)
}

#[test]
fn overloads_share_one_binding_and_preserve_public_diagnostics() {
    for case in ["overloads-tests", "overloads-default-tests"] {
        let module = module(case, "src/api.mts");
        let bindings: Vec<_> = module
            .facts
            .bindings
            .iter()
            .filter(|binding| binding.name == "choose")
            .collect();
        assert_eq!(bindings.len(), 1);
        assert!(!module.complete);
        assert_eq!(module.facts.diagnostics.len(), 1);
        assert_eq!(
            module.facts.diagnostics[0].message,
            "Merged binding declarations are unsupported"
        );
        let span = bindings[0].span;
        assert!(module.facts.importer_safe.contains(&(span.start, span.end)));
    }
}

#[test]
fn function_namespace_merges_are_not_overloads() {
    let module = module("runtime-controls", "src/merged.mts");
    assert!(!module.complete);
    assert!(module.facts.importer_safe.is_empty());
}

#[test]
fn ambient_exceptions_preserve_public_module_completeness() {
    for (case, file, count) in [
        ("ambient", "src/modules.d.ts", 3),
        ("ambient", "src/top.d.ts", 1),
        ("ambient", "src/ambient.mts", 2),
        ("augmentation", "src/augment.mts", 1),
        ("ambient", "src/nested.d.ts", 4),
        ("ambient", "src/global.mts", 1),
    ] {
        let module = module(case, file);
        assert!(!module.complete, "{file}");
        assert_eq!(module.facts.diagnostics.len(), count, "{file}");
        assert_eq!(module.facts.importer_safe.len(), count, "{file}");
    }
}
