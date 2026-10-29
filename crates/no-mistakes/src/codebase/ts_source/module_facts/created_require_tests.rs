use super::*;
use std::path::PathBuf;

fn module(file: &str) -> TypeScriptModule {
    analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/typescript-module-facts"),
        ),
        files: vec![PathBuf::from(file)],
    })
    .unwrap()
    .modules
    .remove(0)
}

// The importer rule reads created loaders privately; the public module facts
// stay exactly what a shadowed `require` produced before.
#[test]
fn created_require_leaves_public_module_facts_unchanged() {
    let shadowed = module("created-require.ts");
    assert!(!shadowed.complete);
    assert!(shadowed.facts.loads.is_empty());
    assert!(shadowed.facts.require_bindings.is_empty());
    let messages: Vec<_> = shadowed
        .facts
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect();
    assert_eq!(messages, ["Shadowed require is not a module loader"]);
    assert_eq!(shadowed.facts.created_requires.len(), 1);
    assert_eq!(shadowed.facts.importer_safe.len(), 1);
}

// CommonJS export diagnostics stay in source order; only the private
// importer-safe list depends on the later whole-file decision.
#[test]
fn commonjs_export_diagnostics_keep_source_order() {
    for file in ["commonjs-exports.cjs", "commonjs-mutated-exports.cjs"] {
        let starts: Vec<_> = module(file)
            .facts
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.span.as_ref().unwrap().start)
            .collect();
        assert!(starts.len() > 1, "{file}");
        assert!(
            starts.windows(2).all(|pair| pair[0] <= pair[1]),
            "{file}: {starts:?}"
        );
    }
}

#[test]
fn renamed_created_loader_adds_no_public_load_or_diagnostic() {
    let renamed = module("created-loader.ts");
    assert!(renamed.complete, "{:?}", renamed.facts.diagnostics);
    assert!(renamed.facts.loads.is_empty());
    assert!(renamed.facts.diagnostics.is_empty());
    assert_eq!(renamed.facts.created_requires.len(), 1);
}
