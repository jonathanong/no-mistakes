use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_supplemental_call_sites, CheckFactPlan,
};
use crate::codebase::ts_source::facts::TsFactPlan;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

#[test]
fn finite_set_call_sources_skip_unrelated_walks_without_changing_findings() {
    let saved = call_literal_fixture_root("scoped-demand");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = crate::codebase::ts_resolver::normalize_path(fixture.path());
    let call = root.join("schedules.mts");
    let registry = root.join("registry.mts");
    let unrelated = root.join("unrelated.tsx");
    let files = vec![call.clone(), registry.clone(), unrelated.clone()];
    let config = call_literal_config("register");
    let requested = try_required_call_site_fact_files(&root, &config).unwrap();
    assert_eq!(requested, [call.clone()]);
    let collect = |broad| {
        let observer = crate::diagnostics::InvocationObserver::new(true);
        let session = AnalysisSession::new(Some(observer.clone()));
        let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
            &files,
        ))));
        let mut plan = CheckFactPlan {
            imports: true,
            symbols: true,
            production_import_paths: Arc::new([call.clone()].into_iter().collect()),
            graph: TsFactPlan {
                imports: true,
                call_sites: broad,
                ..Default::default()
            },
            ..Default::default()
        };
        plan.require_call_site_paths(&requested);
        let facts = crate::diagnostics::with_observer(Some(observer.clone()), || {
            crate::ast::with_request_parse_cache(|| {
                collect_check_facts_with_supplemental_call_sites(
                    &session,
                    &root,
                    (
                        vec![call.clone(), registry.clone()],
                        vec![unrelated.clone()],
                    ),
                    vec![call.clone()],
                    plan,
                    None,
                    sources.clone(),
                )
            })
        });
        let findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts))
                .unwrap();
        (facts, findings, observer.snapshot().work)
    };
    let (scoped, findings, scoped_work) = collect(false);
    let (broad, broad_findings, broad_work) = collect(true);
    assert_eq!(findings, broad_findings);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].target.as_deref(), Some("missing"));
    assert_eq!(
        scoped.ts[&call].ts.call_sites,
        broad.ts[&call].ts.call_sites
    );
    assert_eq!(scoped.ts[&call].ts.call_sites.len(), 2);
    assert!(scoped.ts[&call].production_imports.is_some());
    assert!(!scoped.ts[&call].ts.imports.is_empty());
    assert!(scoped.ts[&registry].ts.symbols.is_some());
    assert!(scoped.ts[&unrelated].ts.call_sites.is_empty());
    assert_eq!(broad.ts[&unrelated].ts.call_sites.len(), 2);
    // An empty registry also incurs a visitor in the broad plan; assert work,
    // since equal findings alone cannot detect the unnecessary traversals.
    assert_eq!(broad_work["ast.walks"], scoped_work["ast.walks"] + 2);
    assert_eq!(broad_work["parse.files"], 3);
    assert_eq!(scoped_work["parse.files"], 3);
}
