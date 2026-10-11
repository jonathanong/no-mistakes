use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::batch::{
    collect_check_fact_batch_with_session, BatchCheckFactRequest,
};
use crate::codebase::ts_source::facts::TsFactPlan;
use std::sync::Arc;

#[test]
fn requested_runner_helpers_keep_union_recovered_facts_and_parse_once() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/importers/malformed-runner-helper");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = AnalysisSession::new(Some(observer.clone()));
    let dataset = session.dataset(&root);
    let visible = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    let catalog = Arc::new(
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources_with_workspace(
            &root,
            std::slice::from_ref(&root),
            &visible,
            &sources,
            &dataset.workspace(),
        ),
    );
    let runners = Arc::new(
        crate::integration_tests::prepare_runner_configs_with_catalog(
            &root,
            &dataset.config(None).unwrap(),
            &visible,
            catalog,
            sources.clone(),
        ),
    );
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible)
        .into_iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .collect::<Vec<_>>();
    let helper = root.join("setup.ts");
    let requests = vec![
        BatchCheckFactRequest {
            root: root.clone(),
            files: files.clone(),
            graph_files: Vec::new(),
            plan: CheckFactPlan {
                integration_runner_configs: Some(runners),
                source: true,
                legacy_symbol_paths: [helper.clone()].into_iter().collect(),
                graph: TsFactPlan::imports_and_symbols(),
                ..Default::default()
            },
            playwright: None,
            sources: sources.clone(),
        },
        BatchCheckFactRequest {
            root: root.clone(),
            files,
            graph_files: Vec::new(),
            plan: CheckFactPlan {
                graph: TsFactPlan::imports_and_symbols(),
                legacy_symbol_paths: [helper.clone()].into_iter().collect(),
                ..Default::default()
            },
            playwright: None,
            sources,
        },
    ];
    crate::ast::begin_parse_count(&root);
    let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::ast::with_request_parse_cache(|| {
            collect_check_fact_batch_with_session(&session, requests)
        })
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 4, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    for facts in &facts {
        let recovered = &facts.ts[&helper];
        assert!(recovered.ts.parse_error.is_some());
        assert_eq!(recovered.ts.imports.len(), 1);
        assert!(!recovered.ts.exported_bindings.is_empty());
        assert!(recovered.legacy_symbols.is_some());
    }
    assert!(facts[0].ts[&helper].source.is_some());
    assert!(facts[1].ts[&helper].source.is_none());
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["parse.files"], 4, "{work:#?}");
    assert!(session
        .work_snapshot()
        .parse_attempts
        .values()
        .all(|attempts| *attempts == 1));
    assert!(observer
        .source_read_snapshot()
        .values()
        .all(|reads| *reads == 1));
}
