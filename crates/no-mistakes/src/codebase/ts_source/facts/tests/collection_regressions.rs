use super::*;

#[test]
fn queue_factory_context_without_glob_matches_all_paths() {
    let ts = fixture("imports.ts");
    let context = TsFactContext::new(ts.parent().unwrap());

    assert!(context.matches_queue_factory(&ts));
}

#[test]
fn collect_ts_facts_skips_non_indexable_files_and_preserves_read_errors() {
    let ts = fixture("imports.ts");
    let txt = fixture("plain.txt");
    let missing = fixture("missing.ts");
    let facts = collect_ts_facts(&[ts.clone(), txt, missing.clone()], TsFactPlan::imports());

    assert_eq!(facts.len(), 2);
    assert_eq!(facts[&ts].imports.len(), 1);
    assert!(facts[&ts].symbols.is_none());
    assert!(facts[&missing]
        .operational_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
    assert!(facts[&missing]
        .parse_error
        .as_deref()
        .is_some_and(|error| error.contains("failed to read")));
}

#[test]
fn collect_ts_facts_uses_tsx_parser_and_symbols_when_requested() {
    let tsx = fixture("component.tsx");
    let facts = collect_ts_facts(
        std::slice::from_ref(&tsx),
        TsFactPlan::imports_and_symbols(),
    );

    assert_eq!(facts[&tsx].imports.len(), 1);
    assert!(facts[&tsx].symbols.is_some());
}

#[test]
fn exported_resource_roots_are_collected_only_for_resource_plans() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/resource-impact/exported-member-consumer.ts");
    let imports_and_symbols = collect_ts_facts(
        std::slice::from_ref(&source),
        TsFactPlan::imports_and_symbols(),
    );
    assert!(imports_and_symbols[&source]
        .exported_resource_roots
        .is_empty());
    assert!(imports_and_symbols[&source]
        .exported_resource_scopes
        .is_empty());

    let resources = collect_ts_facts(
        std::slice::from_ref(&source),
        TsFactPlan {
            imports: true,
            function_calls: true,
            resources: true,
            ..TsFactPlan::default()
        },
    );
    assert_eq!(
        resources[&source].exported_resource_roots,
        ["NamedService", "Service", "api", "default", "eagerApi"]
    );
    assert!(resources[&source]
        .exported_resource_scopes
        .iter()
        .any(|scope| scope == "api/nested/load"));
}

#[test]
fn collect_ts_facts_can_skip_import_collection() {
    let ts = fixture("imports.ts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&ts),
        TsFactPlan {
            imports: false,
            symbols: false,
            ..TsFactPlan::default()
        },
    );

    assert!(facts[&ts].imports.is_empty());
    assert!(facts[&ts].symbols.is_none());
}

#[test]
fn collect_file_facts_falls_back_to_ts_source_type_for_unknown_extension() {
    let unknown = fixture("unknown-extension.source");
    let facts = collect_file_facts(&unknown, TsFactPlan::imports(), &TsFactContext::default())
        .expect("unknown extension fixture should still parse as TypeScript");

    assert_eq!(facts.imports.len(), 1);
}

#[path = "../tests_observer.rs"]
mod observer_tests;
