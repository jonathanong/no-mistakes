#[test]
fn plan_constructors_select_expected_fact_sets() {
    let imports = TsFactPlan::imports();
    assert!(imports.imports);
    assert!(!imports.symbols);

    let both = TsFactPlan::imports_and_symbols();
    assert!(both.imports);
    assert!(both.symbols);
}

#[test]
fn call_site_facts_are_collected_only_when_requested() {
    let file = fixture("imports.ts");
    let without_call_sites = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan::imports_and_symbols(),
    );
    assert!(without_call_sites[&file].call_sites.is_empty());

    let with_call_sites = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            call_sites: true,
            ..TsFactPlan::imports_and_symbols()
        },
    );
    let call_site = with_call_sites[&file]
        .call_sites
        .iter()
        .find(|site| site.callee == "helper")
        .expect("helper call site");
    assert_eq!(call_site.line, 3);
    assert_eq!(call_site.arg_count, 0);
    assert!(call_site.caller.is_none());
}

#[test]
fn source_facts_preserve_owned_public_api_and_reuse_physical_read() {
    let file = fixture("imports.ts");
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let expected = sources.read_path(&file).unwrap();

    let mut facts = super::collect::collect_ts_facts_with_context_and_sources(
        std::slice::from_ref(&file),
        TsFactPlan {
            source: true,
            ..TsFactPlan::default()
        },
        &TsFactContext::default(),
        &sources,
    );

    let source: &String = facts[&file].source.as_ref().unwrap();
    assert_eq!(source, expected.as_ref());
    let symbols: Option<crate::codebase::ts_symbols::FileSymbols> = facts[&file].symbols.clone();
    assert!(symbols.is_none());
    let components: &mut Vec<crate::react_traits::report::types::ComponentFacts> =
        &mut facts.get_mut(&file).unwrap().react_components;
    components.clear();
    let owned: Vec<(PathBuf, TsFileFacts)> = facts.into_iter().collect();
    assert_eq!(owned.len(), 1);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn empty_serial_paths_collect_symbols_with_the_parallel_fact_path() {
    let file = fixture("imports.ts");
    let inventory = std::sync::Arc::new(crate::codebase::ts_source::FileInventory::from_paths(
        std::slice::from_ref(&file),
    ));
    let sources = crate::codebase::ts_source::SourceStore::new(inventory);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();

    let facts = super::collect::collect_ts_facts_with_context_sources_and_session_serializing_paths(
        &session,
        std::slice::from_ref(&file),
        TsFactPlan::imports_and_symbols(),
        &TsFactContext::default(),
        &sources,
        &[],
    );

    assert!(facts[&file].symbols.is_some());
    assert_eq!(facts[&file].imports.len(), 1);
    assert_eq!(sources.physical_read_count(), 1);
}
