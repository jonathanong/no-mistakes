use super::*;
use std::path::PathBuf;
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/typescript-module-facts")
}
fn report(files: &[&str]) -> TypeScriptModulesReport {
    analyze_typescript_modules(&TypeScriptModulesOptions {
        root: Some(root()),
        files: files.iter().map(PathBuf::from).collect(),
    })
    .unwrap()
}
#[test]
fn modules_preserve_bindings_shadowing_types_and_import_forms() {
    let result = report(&["bindings.ts", "bindings.ts"]);
    assert_eq!(result.modules.len(), 1);
    let module = &result.modules[0];
    assert!(module.complete, "{:?}", module.facts.diagnostics);
    assert_eq!(module.facts.imports.len(), 4);
    assert_eq!(module.facts.exports.len(), 8);
    assert_eq!(module.facts.loads.len(), 2);
    let imported = module
        .facts
        .bindings
        .iter()
        .find(|binding| binding.name == "alias" && binding.imported)
        .unwrap();
    assert!(imported.runtime);
    assert!(imported
        .references
        .iter()
        .any(|reference| reference.runtime));
    assert_eq!(
        module
            .facts
            .bindings
            .iter()
            .filter(|binding| binding.shadows == Some(imported.id))
            .count(),
        2
    );
    let only_type = module
        .facts
        .bindings
        .iter()
        .find(|binding| binding.name == "onlyType")
        .unwrap();
    assert!(only_type
        .references
        .iter()
        .all(|reference| !reference.runtime && reference.type_only));
    let shape = module
        .facts
        .bindings
        .iter()
        .find(|binding| binding.name == "Shape")
        .unwrap();
    assert!(shape.type_only);
    assert!(!shape.runtime);
    let source = std::fs::read(root().join("bindings.ts")).unwrap();
    for binding in &module.facts.bindings {
        assert_eq!(
            std::str::from_utf8(&source[binding.span.start as usize..binding.span.end as usize])
                .unwrap(),
            binding.name
        );
    }
}
#[test]
fn modules_fail_closed_for_dynamic_invalid_missing_and_unsupported_sources() {
    let result = report(&[
        "dynamic.ts",
        "invalid.ts",
        "missing.ts",
        "unsupported.txt",
        "empty.ts",
    ]);
    for module in &result.modules {
        if module.file_name.ends_with("empty.ts") {
            assert!(module.complete);
        } else {
            assert!(!module.complete);
            assert!(!module.facts.diagnostics.is_empty());
        }
        if module.file_name.ends_with("dynamic.ts") {
            assert!(module.facts.loads.is_empty());
            assert_eq!(module.facts.diagnostics.len(), 7);
        }
    }
    assert!(report(&[]).modules.is_empty());
}
#[test]
fn module_fact_demands_are_unionable_and_cached() {
    use crate::codebase::ts_source::facts::TsFactPlan;
    let mut plan = TsFactPlan::default();
    let demand = TsFactPlan {
        module_bindings: true,
        ..TsFactPlan::default()
    };
    assert!(!demand.is_empty());
    assert!(!plan.covers(demand));
    plan.include(demand);
    assert!(plan.covers(demand));
}

#[test]
fn additional_export_forms_and_semantic_gaps_preserve_spans() {
    let result = report(&[
        "default-forms.ts",
        "default-anonymous.ts",
        "default-function.ts",
        "default-class.ts",
        "default-interface.ts",
        "default-expression.ts",
        "legacy.ts",
        "dynamic-with.cjs",
        "semantic-invalid.ts",
        "unicode.ts",
        "indirect.ts",
        "declaration-merge.ts",
        "literal-loads.ts",
    ]);
    for module in result.modules {
        if [
            "legacy.ts",
            "dynamic-with.cjs",
            "semantic-invalid.ts",
            "indirect.ts",
            "declaration-merge.ts",
        ]
        .iter()
        .any(|name| module.file_name.ends_with(name))
        {
            assert!(!module.complete, "{}", module.file_name.display());
        } else {
            assert!(module.complete, "{:?}", module.facts.diagnostics);
        }
        if module.file_name.ends_with("literal-loads.ts") {
            assert_eq!(module.facts.loads.len(), 4);
        }
        if module.file_name.ends_with("unicode.ts") {
            let source = std::fs::read(&module.file_name).unwrap();
            let binding = &module.facts.bindings[0];
            assert_eq!(
                std::str::from_utf8(
                    &source[binding.span.start as usize..binding.span.end as usize]
                )
                .unwrap(),
                "café"
            );
        }
        if module.file_name.ends_with("default-forms.ts") {
            assert!(module
                .facts
                .exports
                .iter()
                .any(|entry| entry.exported == "ClassValue"));
            let ambient = module
                .facts
                .bindings
                .iter()
                .find(|binding| binding.name == "ambient")
                .unwrap();
            assert!(!ambient.runtime);
        }
    }
    let empty = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: None,
        files: vec![],
    })
    .unwrap();
    assert!(empty.modules.is_empty());
    let absolute = analyze_typescript_modules(&TypeScriptModulesOptions {
        root: None,
        files: vec![root().join("empty.ts")],
    })
    .unwrap();
    assert!(absolute.modules[0].complete);
}

#[test]
fn selected_module_demand_shares_one_source_read_and_parse_with_existing_facts() {
    use crate::codebase::ts_source::facts::{TsFactContext, TsFactPlan};
    use crate::codebase::ts_source::{FileInventory, SourceStore};
    use std::sync::Arc;
    let path = crate::codebase::ts_resolver::normalize_path(&root().join("bindings.ts"));
    let files = vec![path.clone(), path.clone()];
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let sources =
        SourceStore::new_observed(Arc::new(FileInventory::from_paths(&files)), Some(observer));
    let mut plan = TsFactPlan::imports_and_symbols();
    plan.include(TsFactPlan {
        module_bindings: true,
        ..TsFactPlan::default()
    });
    let facts =
        crate::codebase::ts_source::facts::collect_ts_facts_with_context_sources_and_session(
            &session,
            &files,
            plan,
            &TsFactContext::default(),
            &sources,
        );
    assert_eq!(facts.len(), 1);
    assert!(!facts[&path].imports.is_empty());
    assert!(facts[&path].symbols.is_some());
    assert!(facts[&path].module_bindings.is_some());
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(session.work_snapshot().parse_attempts[&path], 1);
}
