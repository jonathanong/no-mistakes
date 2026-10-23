use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::batch::{
    collect_check_fact_batch_with_session, BatchCheckFactRequest,
};
use crate::codebase::ts_source::facts::TsFactPlan;
use std::sync::Arc;

#[test]
fn runner_graph_helpers_retain_scoped_call_sites_and_richer_imports() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/finite-set-consistency/call-literals/scoped-demand");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    for batch in [false, true] {
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
        let helper = root.join("setup.ts");
        let config = root.join("vitest.config.ts");
        let mut plan = CheckFactPlan {
            integration_runner_configs: Some(runners),
            graph: TsFactPlan::imports_and_symbols(),
            ..Default::default()
        };
        plan.require_call_site_paths(std::slice::from_ref(&helper));
        let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::ast::with_request_parse_cache(|| {
                if batch {
                    collect_check_fact_batch_with_session(
                        &session,
                        vec![BatchCheckFactRequest {
                            root: root.clone(),
                            files: vec![config.clone()],
                            graph_files: vec![helper.clone()],
                            plan,
                            playwright: None,
                            sources: sources.clone(),
                        }],
                    )
                    .pop()
                    .unwrap()
                } else {
                    crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
                        &session, &root, (vec![config.clone()], vec![helper.clone()]),
                        plan, None, sources.clone(),
                    )
                }
            })
        });
        let file = &facts.ts[&helper];
        assert_eq!(file.ts.call_sites.len(), 1, "batch={batch}");
        assert_eq!(file.ts.call_sites[0].callee, "configure");
        assert!(file.ts.symbols.is_some());
        assert!(!file.ts.exported_bindings.is_empty());
        assert_eq!(facts.ts[&config].ts.imports.len(), 1);
        assert!(facts.ts[&config].integration_runner_config.is_some());
        assert!(session
            .work_snapshot()
            .parse_attempts
            .values()
            .all(|count| *count == 1));
    }
}
