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
    let payload = facts[&path]
        .module_bindings
        .as_ref()
        .unwrap()
        .bindings
        .as_ptr();
    let report = super::report::project_modules(vec![path.clone()], facts);
    assert_eq!(report.modules[0].facts.bindings.as_ptr(), payload);
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(session.work_snapshot().parse_attempts[&path], 1);
}

#[test]
fn wrapped_direct_eval_is_an_explicit_binding_gap() {
    let module = report(&["wrapped-eval.ts"]).modules.remove(0);
    assert!(!module.complete);
    assert_eq!(module.facts.diagnostics.len(), 5);
    assert!(module
        .facts
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.message == "Dynamic eval may change binding semantics"));
}

#[test]
fn public_boundary_records_source_and_fact_work() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        assert!(report(&["bindings.ts", "bindings.ts"]).modules[0].complete);
    });
    let snapshot = observer.snapshot();
    assert_eq!(snapshot.work["source.reads"], 1);
    assert_eq!(snapshot.work["ts_facts.collections"], 1);
    assert_eq!(snapshot.work["ts_facts.files"], 1);
}

#[test]
fn type_only_declarations_and_wrapped_default_exports_keep_binding_links() {
    let modules = report(&[
        "all-type-imports.ts",
        "default-wrapped-as.ts",
        "default-wrapped-nonnull.ts",
        "default-wrapped-parenthesis.ts",
        "default-wrapped-satisfies.ts",
    ])
    .modules;
    for module in modules {
        assert!(module.complete, "{:?}", module.facts.diagnostics);
        if module.file_name.ends_with("all-type-imports.ts") {
            assert!(module.facts.imports[0].type_only);
            assert!(module.facts.imports[0]
                .bindings
                .iter()
                .all(|binding| binding.type_only));
            // An empty named import still evaluates its source module.
            assert!(!module.facts.imports[1].type_only);
        } else {
            assert_eq!(module.facts.exports[0].local, "value");
        }
    }
}

#[test]
fn shadow_bindings_skip_empty_intermediate_scopes() {
    let module = report(&["skipped-shadow-scope.ts"]).modules.remove(0);
    assert!(module.complete);
    for name in ["x", "outerName"] {
        let bindings = module
            .facts
            .bindings
            .iter()
            .filter(|binding| binding.name == name)
            .collect::<Vec<_>>();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].shadows, None);
        assert_eq!(bindings[1].shadows, Some(bindings[0].id));
    }
    assert_eq!(
        module
            .facts
            .bindings
            .iter()
            .find(|binding| binding.name == "neverShadowed")
            .unwrap()
            .shadows,
        None
    );
}

#[test]
fn local_export_types_wrapped_loads_and_commonjs_gaps_are_explicit() {
    for module in report(&[
        "local-type-exports.ts",
        "shadowed-eval.cjs",
        "wrapped-require.ts",
        "commonjs-exports.cjs",
        "local-export-objects.cjs",
    ])
    .modules
    {
        if module.file_name.ends_with("commonjs-exports.cjs") {
            assert!(!module.complete);
            assert_eq!(module.facts.diagnostics.len(), 5);
        } else {
            assert!(module.complete, "{:?}", module.facts.diagnostics);
        }
        if module.file_name.ends_with("wrapped-require.ts") {
            assert_eq!(module.facts.loads.len(), 6);
        }
        if module.file_name.ends_with("local-type-exports.ts") {
            assert!(module.facts.exports[..3]
                .iter()
                .all(|export| export.type_only));
            assert!(!module.facts.exports[3].type_only);
            // Source re-exports remain syntactic, without resolving another module.
            assert!(!module.facts.exports[4].type_only);
        }
    }
}

#[test]
fn source_only_exports_optional_eval_and_ambient_ownership_are_preserved() {
    for module in report(&[
        "empty-source-exports.ts",
        "optional-eval.cjs",
        "ambient-require.cts",
        "ambient-indirect-require.cts",
        "ambient-modules.d.ts",
    ])
    .modules
    {
        if module.file_name.ends_with("ambient-modules.d.ts") {
            assert!(!module.complete);
            assert_eq!(module.facts.exports.len(), 1);
            assert_eq!(module.facts.exports[0].exported, "rootValue");
            assert!(module.facts.imports.is_empty());
        } else if module.file_name.ends_with("ambient-indirect-require.cts") {
            assert!(!module.complete);
            assert!(module.facts.diagnostics.iter().any(
                |diagnostic| diagnostic.message == "Indirect require reference is unsupported"
            ));
        } else {
            assert!(module.complete, "{:?}", module.facts.diagnostics);
        }
        if module.file_name.ends_with("empty-source-exports.ts") {
            assert_eq!(module.facts.exports.len(), 2);
            assert_eq!(
                module.facts.exports[0].specifier.as_deref(),
                Some("./runtime")
            );
            assert_eq!(module.facts.exports[0].exported, "");
            assert!(!module.facts.exports[0].type_only);
            assert!(module.facts.exports[1].type_only);
        }
        if module.file_name.ends_with("ambient-require.cts") {
            assert_eq!(module.facts.loads.len(), 2);
        }
    }
}

#[test]
fn commonjs_object_uses_and_global_defaults_do_not_claim_complete_local_exports() {
    for module in report(&[
        "commonjs-property-exports.cjs",
        "commonjs-update-exports.cjs",
        "ambient-commonjs-exports.cts",
        "commonjs-types.cts",
        "default-global.ts",
        "default-undefined.ts",
        "default-wrapped-global.ts",
    ])
    .modules
    {
        if module.file_name.ends_with("commonjs-property-exports.cjs") {
            assert!(!module.complete);
            assert_eq!(module.facts.diagnostics.len(), 5);
        } else if module.file_name.ends_with("commonjs-update-exports.cjs") {
            assert!(!module.complete);
            assert_eq!(module.facts.diagnostics.len(), 3);
        } else if module.file_name.ends_with("ambient-commonjs-exports.cts") {
            assert!(!module.complete);
            assert_eq!(module.facts.diagnostics.len(), 2);
        } else {
            assert!(module.complete, "{:?}", module.facts.diagnostics);
        }
        if module
            .file_name
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("default-")
        {
            assert_eq!(module.facts.exports[0].local, "");
        }
    }
}
