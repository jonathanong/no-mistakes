use super::*;

pub(super) fn assert_prepared_matches_standalone(
    root: &Path,
    config: &NoMistakesConfig,
    count: usize,
) {
    let standalone = check(root, config, None).unwrap();
    let plan = graph_plan(config).unwrap();
    let (graph, graph_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan(root, plan);
    let files = crate::codebase::ts_source::discover_files(root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            graph,
            graph_context,
            ..Default::default()
        },
    );
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(observer.clone()));
    session.visible_paths(root);
    let tsconfig = resolve_tsconfig(root, None).unwrap();
    let prepared = shared_facts::check_with_prepared_facts_and_session(
        root, config, None, &tsconfig, &facts, None, &session,
    )
    .unwrap();
    assert_eq!(prepared, standalone);
    assert_eq!(prepared.len(), count);
    let work = observer.snapshot().work;
    assert_eq!(work["discovery.roots"], 1, "{work:#?}");
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert_eq!(
        work.get("parse.files").copied().unwrap_or_default(),
        0,
        "{work:#?}"
    );
}
