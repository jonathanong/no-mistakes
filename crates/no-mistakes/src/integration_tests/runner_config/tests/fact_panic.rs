use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};

#[test]
fn fact_panic_in_prepared_runner_helper_is_promoted_to_new_check_session() {
    let saved = fixture_root("basic");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let path = root.join("vitest.config.mts");
    let prepared = prepare_vitest(&root, StringOrList::One("vitest.config.mts".into()));
    let primary_plan = CheckFactPlan {
        imports: true,
        integration_runner_configs: Some(Arc::new(prepared.clone())),
        ..Default::default()
    };
    let mut collection_plan = primary_plan.clone();
    Arc::make_mut(&mut collection_plan.production_import_paths).insert(path.clone());
    let fact_plan = RunnerConfigFactPlan {
        integration_discovery: crate::codebase::check_facts::FamilyPathScope::All,
        root: root.clone(),
        primary_files: [path.clone()].into(),
        graph_files: Default::default(),
        primary_plan,
        graph_plan: Default::default(),
        playwright: None,
    };
    let source = std::fs::read_to_string(&path).unwrap();
    let (_, mut helpers) = crate::codebase::fact_panic::tests::with_injected_panic(&path, || {
        prepared.with_request_cache(Some(fact_plan), || {
            with_program(&path, &source, |_, _| ()).unwrap()
        })
    });
    let failed = helpers
        .remove(&path)
        .expect("prepared helper records failure");
    assert!(failed.fact_extraction_panicked);
    assert!(!failed.parsed);
    assert!(failed.ts.imports.is_empty());
    assert!(failed
        .ts
        .operational_error
        .as_ref()
        .unwrap()
        .contains("injected fact extraction failure"));
    let mut facts = collect_check_facts(&root, vec![path.clone()], CheckFactPlan::default());
    facts.ts.insert(path.clone(), Arc::new(failed));
    let session = AnalysisSession::disabled();
    facts.register_fact_panics(&session, facts.ts.keys());
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].consumer, "fact-extraction");
    assert_eq!(failures[0].file.as_ref(), Some(&path));
    assert!(failures[0]
        .message
        .contains("injected fact extraction failure"));
    let facts = crate::codebase::fact_panic::tests::with_injected_panic(&path, || {
        crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_session(
            &session,
            &root,
            vec![path.clone()],
            vec![],
            collection_plan,
            None,
        )
    });
    let failed = &facts.ts[&path];
    assert!(failed.fact_extraction_panicked);
    assert!(failed.production_imports.is_none());
    assert!(failed.ts.imports.is_empty());
}
