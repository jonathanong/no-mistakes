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
fn call_site_facts_preserve_optional_identifier_calls() {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/queries/optional-identifier-call/consumer.ts");
    let facts = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            call_sites: true,
            ..TsFactPlan::default()
        },
    );

    let call_site = facts[&file]
        .call_sites
        .iter()
        .find(|site| site.callee == "used")
        .expect("optional used call site");
    assert!(call_site.is_optional);
}

#[test]
fn call_site_facts_preserve_optional_member_calls() {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/rules/finite-set-consistency/call-literals/optional-chain/schedules.mts",
    );
    let facts = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            call_sites: true,
            ..TsFactPlan::default()
        },
    );

    let sites = &facts[&file].call_sites;
    assert_eq!(sites.len(), 2, "{sites:?}");
    assert!(sites.iter().all(|site| site.is_optional));
}

#[test]
fn call_site_facts_render_one_level_this_member_calls() {
    let file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../fixtures/rules/finite-set-consistency/call-literals/this-receiver/schedules.mts",
    );
    let facts = collect_ts_facts(
        std::slice::from_ref(&file),
        TsFactPlan {
            call_sites: true,
            ..TsFactPlan::default()
        },
    );

    let call_site = facts[&file]
        .call_sites
        .iter()
        .find(|site| site.callee == "this.register")
        .expect("this member call site");
    assert_eq!(call_site.static_arg_source.as_deref(), Some("\"job\""));
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

    let source: &std::sync::Arc<str> = facts[&file].source.as_ref().unwrap();
    assert!(std::sync::Arc::ptr_eq(source, &expected));
    assert!(facts[&file].symbols.is_none());
    std::sync::Arc::make_mut(&mut facts.get_mut(&file).unwrap().react_components).clear();
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

#[test]
fn collect_domain_facts_fuses_http_and_effect_call_walks() {
    let source = include_str!("../domain.rs");
    assert!(
        source.contains("collect_fused_domain_calls"),
        "domain facts must walk HTTP and effect calls together"
    );
    assert!(
        !source.contains("extract_http_calls_from_program")
            && !source.contains("effect_calls::extract"),
        "HTTP and effect extractors must not walk the program again"
    );
}

#[test]
fn trpc_facts_follow_router_globs_and_call_plan() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/trpc-basic/fixture");
    let router = root.join("src/router.ts");
    let client = root.join("src/client.ts");
    let mut context = TsFactContext::new(&root);
    let mut builder = globset::GlobSetBuilder::new();
    builder.add(globset::Glob::new("src/router.ts").unwrap());
    context.trpc_router_glob = Some(builder.build().unwrap());

    let enabled = collect_ts_facts_with_context(
        &[router.clone(), client.clone()],
        TsFactPlan {
            trpc_router: true,
            trpc_calls: true,
            ..TsFactPlan::default()
        },
        &context,
    );
    assert!(enabled[&router]
        .trpc_procedures
        .iter()
        .any(|procedure| procedure == "user.get"));
    assert!(enabled[&client]
        .trpc_calls
        .iter()
        .any(|call| call.path == "user.get"));

    let skipped_router = collect_ts_facts_with_context(
        std::slice::from_ref(&router),
        TsFactPlan {
            trpc_router: true,
            ..TsFactPlan::default()
        },
        &TsFactContext::new(&root),
    );
    assert!(skipped_router[&router].trpc_procedures.is_empty());

    let skipped_calls = collect_ts_facts_with_context(
        std::slice::from_ref(&client),
        TsFactPlan {
            trpc_router: true,
            ..TsFactPlan::default()
        },
        &context,
    );
    assert!(skipped_calls[&client].trpc_calls.is_empty());
}
