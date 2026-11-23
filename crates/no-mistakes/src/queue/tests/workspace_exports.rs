use super::{analyze_project, fixture};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session,
    configure_family_path_scopes, CheckFactPlan,
};
use crate::codebase::ts_resolver::TsConfigCatalog;
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::queue::analyze_project_with_prepared_facts_and_catalog_and_session;

#[test]
fn worker_queue_names_resolve_through_workspace_exports_wildcards() {
    let root = fixture("workspace-exports-3953");
    let automatic = analyze_project(&root, None, &[]).unwrap();
    let explicit = analyze_project(&root, Some(&root.join("tsconfig.json")), &[]).unwrap();

    for report in [&automatic, &explicit] {
        assert!(report.check.is_empty(), "{:#?}", report.check);
        assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
        assert_eq!(report.workers.len(), 2, "{:#?}", report.workers);
        assert!(report.workers.iter().all(|w| w.queue_name.is_some()));
        // Fixed-queueName wrappers without a queue declaration use the deterministic
        // virtual `queue:<name>` anchor rather than an arbitrary caller file.
        assert!(
            report
                .edges
                .iter()
                .any(|e| e.from == "apps/api/send.mts" && e.to == "queue:mail#welcome"),
            "{:#?}",
            report.edges
        );
        assert!(
            report
                .edges
                .iter()
                .any(|e| e.from == "queue:mail#welcome" && e.to == "apps/workers/mail-worker.mts"),
            "{:#?}",
            report.edges
        );
        assert!(
            report
                .edges
                .iter()
                .any(|e| e.from == "apps/alpha/billing.mts" && e.to == "queue:billing#charge"),
            "{:#?}",
            report.edges
        );
        assert!(
            report
                .edges
                .iter()
                .any(|e| e.from == "apps/zeta/billing.mts" && e.to == "queue:billing#charge"),
            "{:#?}",
            report.edges
        );
        assert!(
            report
                .edges
                .iter()
                .all(|e| !e.to.starts_with("apps/alpha/")),
            "billing queue must not anchor on an unrelated caller file: {:#?}",
            report.edges
        );
    }

    let snapshot = serde_json::to_string_pretty(&serde_json::json!({
        "jobs": automatic.jobs,
        "edges": automatic.edges,
        "workers": automatic.workers,
    }))
    .unwrap();
    insta::assert_snapshot!(snapshot);
}

#[test]
fn prepared_catalog_queue_analysis_resolves_workspace_exports_wildcards() {
    let root = fixture("workspace-exports-3953").canonicalize().unwrap();
    let visible = VisiblePathSnapshot::new(&root);
    let session = AnalysisSession::new(None);
    session.insert_visible_paths(&root, std::sync::Arc::new(visible));
    let dataset = session.dataset(&root);
    let files = dataset.paths_for(&root);
    let sources = dataset.sources_for(&root);
    let config = dataset.config(None).unwrap();
    let mut plan = CheckFactPlan {
        queue: true,
        queue_factory_names: config.queues.factories.queue_names(),
        queue_factory_roles: config
            .queues
            .factories
            .clone()
            .with_project_wait_scopes(&config),
        ..Default::default()
    };
    configure_family_path_scopes(&root, &files, &config, &sources, &mut plan).unwrap();
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (files.to_vec(), Vec::new()),
        plan,
        None,
        sources,
    );
    let catalog = TsConfigCatalog::from_visible(&root, &[], &files);
    let report = analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &config,
        &[],
        &facts,
        &session,
    )
    .unwrap();
    assert!(report.check.is_empty(), "{:#?}", report.check);
    assert!(report.diagnostics.is_empty(), "{:#?}", report.diagnostics);
    assert!(
        report
            .edges
            .iter()
            .any(|e| e.from == "queue:mail#welcome" && e.to == "apps/workers/mail-worker.mts"),
        "{:#?}",
        report.edges
    );
}
