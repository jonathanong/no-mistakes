use super::*;
use std::path::PathBuf;

fn module(file: &str) -> TypeScriptModule {
    analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-exports-only-used-by-tests/fixture/static-dynamic-facts")),
        files: vec![PathBuf::from(file)],
    })
    .unwrap()
    .modules
    .remove(0)
}

#[test]
fn static_dynamic_constants_preserve_public_module_facts() {
    let module = module("constant.mts");
    assert!(!module.complete);
    assert_eq!(module.facts.folded_dynamic_imports.len(), 8);
    assert_eq!(module.facts.importer_safe.len(), 8);
    assert_eq!(module.facts.diagnostics.len(), 8);
    assert_eq!(module.facts.loads.len(), 2);
    assert!(module
        .facts
        .diagnostics
        .iter()
        .all(|diagnostic| { diagnostic.message == "Non-literal dynamic import specifier" }));
    let serialized = serde_json::to_value(&module.facts).unwrap();
    let mut public_only = module.facts.clone();
    public_only.folded_dynamic_imports.clear();
    public_only.importer_safe.clear();
    assert_eq!(serialized, serde_json::to_value(public_only).unwrap());
    let folded = &module.facts.folded_dynamic_imports;
    assert_eq!(folded[0].load.specifier, "./target.mjs");
    assert_eq!(folded[1].load.specifier, "./target.mjs");
    assert!(folded[3].relative_url);
    assert!(folded[5].pathname);
    assert_eq!(folded[7].load.specifier, "../target.mjs");
}

#[test]
fn static_dynamic_runtime_controls_remain_uncertain() {
    for file in ["runtime.mts", "depth.mts"] {
        let module = module(file);
        assert!(!module.complete, "{file}");
        assert!(module.facts.folded_dynamic_imports.is_empty(), "{file}");
        assert!(module.facts.importer_safe.is_empty(), "{file}");
        assert!(module.facts.loads.is_empty(), "{file}");
        assert!(!module.facts.diagnostics.is_empty(), "{file}");
    }
}

#[test]
fn static_dynamic_discarded_results_select_no_exports_without_hiding_escapes() {
    let facts = module("discarded.mts").facts;
    assert_eq!(facts.folded_dynamic_imports.len(), 6);
    let selections: Vec<_> = facts
        .dynamic_import_selectors
        .iter()
        .map(|(_, names, _)| names)
        .collect();
    assert_eq!(selections.len(), 4);
    assert_eq!(
        selections.iter().filter(|names| names.is_empty()).count(),
        3
    );
    assert_eq!(selections.last().unwrap().as_slice(), ["selected"]);
}
