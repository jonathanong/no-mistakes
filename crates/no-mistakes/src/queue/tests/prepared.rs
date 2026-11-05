use super::*;

#[test]
fn basic_project_reports_queue_edges() {
    let report = analyze_project(&fixture("basic"), None, &[]).unwrap();
    assert_eq!(report.check, vec![]);
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "enqueue.ts" && edge.to == "queues.ts#sendWelcome"));
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "queues.ts#sendWelcome" && edge.to == "worker.ts"));
}

#[test]
fn indexed_prepared_facts_report_matches_plain_prepared_facts_report() {
    let root = fixture("basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            ..Default::default()
        },
    );
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, facts.files())
            .unwrap();

    let plain = analyze_project_with_prepared_facts(&root, &tsconfig, &[], &facts).unwrap();
    let indexed =
        analyze_project_with_prepared_facts_indexed(&root, &tsconfig, &[], &facts).unwrap();

    assert_eq!(indexed.report().producers, plain.producers);
    assert_eq!(indexed.report().workers, plain.workers);
    assert_eq!(indexed.report().jobs, plain.jobs);
    assert_eq!(indexed.report().edges, plain.edges);
    assert_eq!(indexed.report().diagnostics, plain.diagnostics);
    assert_eq!(indexed.report().check, plain.check);
}

#[test]
fn aggregate_queue_check_reports_selected_parse_failures_without_negative_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("partial-failures"));
    let visible = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let files = visible.paths_for(&root);
    let sources = visible.source_store_for(&root);
    let config = crate::config::v2::load_v2_config_from_visible(&root, None, &files).unwrap();
    let mut plan = crate::codebase::check_facts::CheckFactPlan {
        queue: true,
        queue_factory_names: config.queues.factories.queue_names(),
        queue_factory_roles: config
            .queues
            .factories
            .clone()
            .with_project_wait_scopes(&config),
        ..Default::default()
    };
    crate::codebase::check_facts::configure_family_path_scopes(
        &root, &files, &config, &sources, &mut plan,
    )
    .unwrap();
    let session = crate::codebase::analysis_session::AnalysisSession::new(None);
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (files.to_vec(), Vec::new()),
        plan,
        None,
        sources,
    );
    let broken = root.join("src/broken.ts");
    let unrelated = root.join("src/unrelated.ts");
    let queue_inputs = facts.queue_source_files().unwrap();
    assert!(
        queue_inputs.contains(&broken),
        "configured factory source stays selected"
    );
    assert!(
        !queue_inputs.contains(&unrelated),
        "unrelated malformed source is outside queue demand"
    );
    assert!(
        facts.ts[&broken].parse_error.is_some(),
        "selected source must have a parser failure"
    );
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(&root, &[], &files);

    assert!(analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &[],
        &facts,
        &session,
    )
    .is_err());
    session.enable_partial_checks();
    let report = analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &[],
        &facts,
        &session,
    )
    .unwrap();
    assert!(
        report.check.is_empty(),
        "a failed queue prerequisite cannot support negative queue conclusions"
    );
    let failures = session.check_failures();
    assert!(failures.iter().any(|failure| {
        failure.consumer == "queues"
            && failure
                .file
                .as_deref()
                .is_some_and(|path| path.ends_with("src/broken.ts"))
    }));
    assert!(!failures.iter().any(|failure| {
        failure
            .file
            .as_deref()
            .is_some_and(|path| path.ends_with("src/unrelated.ts"))
    }));
}
