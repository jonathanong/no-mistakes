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

#[test]
fn configured_test_scopes_excluded_and_same_file_workers_match() {
    let root = fixture("test-scope-and-same-file-workers");
    let standalone = analyze_project(&root, None, &[]).unwrap();

    let canonical = root.canonicalize().unwrap();
    let visible = VisiblePathSnapshot::new(&canonical);
    let session = AnalysisSession::new(None);
    session.insert_visible_paths(&canonical, std::sync::Arc::new(visible));
    let dataset = session.dataset(&canonical);
    let files = dataset.paths_for(&canonical);
    let sources = dataset.sources_for(&canonical);
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
    configure_family_path_scopes(&canonical, &files, &config, &sources, &mut plan).unwrap();
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &canonical,
        (files.to_vec(), Vec::new()),
        plan,
        None,
        sources,
    );
    let catalog = TsConfigCatalog::from_visible(&canonical, &[], &files);
    let prepared = analyze_project_with_prepared_facts_and_catalog_and_session(
        &canonical,
        &catalog,
        &config,
        &[],
        &facts,
        &session,
    )
    .unwrap();

    for report in [&standalone, &prepared] {
        assert_eq!(report.check.len(), 1, "{:#?}", report.check);
        assert_eq!(report.check[0].kind, "unmatched-producer");
        assert_eq!(report.check[0].file, "apps/api/orphan-enqueue.mts");
        assert_eq!(report.check[0].job.as_deref(), Some("never-handled"));
        assert!(
            report
                .edges
                .iter()
                .any(|e| e.from == "apps/api/send-mail.mts"
                    && e.to == "packages/queues-mail/config.mts#welcome"),
            "{:#?}",
            report.edges
        );
        assert!(
            report
                .edges
                .iter()
                .any(|e| e.from == "packages/queues-mail/config.mts#welcome"
                    && e.to == "apps/workers/mail-worker.mts"),
            "{:#?}",
            report.edges
        );
        assert!(
            report
                .edges
                .iter()
                .all(|e| { !e.from.contains("/integration/") && !e.to.contains("/integration/") }),
            "configured test scopes must not leak into production queues edges: {:#?}",
            report.edges
        );
    }
}

#[test]
fn unconfigured_test_filenames_are_not_excluded_by_convention_and_same_file_workers_match() {
    let root = fixture("no-test-scope-convention");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        report
            .check
            .iter()
            .all(|f| f.file != "same-file-raw.test.mts"),
        "same-file raw/factory workers must match without configured test scope: {:#?}",
        report.check
    );
    assert_eq!(report.check.len(), 1, "{:#?}", report.check);
    assert_eq!(report.check[0].kind, "unmatched-producer");
    assert_eq!(report.check[0].file, "smoke.test.mts");
    assert_eq!(report.check[0].job.as_deref(), Some("unhandled-smoke"));
}

#[test]
fn configured_test_scope_matches_playwright_globs_and_stays_empty_without_config() {
    let root = fixture("no-test-scope-convention").canonicalize().unwrap();
    let session = AnalysisSession::new(None);
    let visible = VisiblePathSnapshot::new(&root);
    let paths = visible.paths_for(&root);
    session.insert_visible_paths(&root, std::sync::Arc::new(visible));

    let empty = crate::queue::test_scope::ConfiguredTestScope::empty();
    assert!(!empty.is_match(&root, &root.join("smoke.test.mts")));

    let default_cfg = crate::config::v2::NoMistakesConfig::default();
    let unconfigured = crate::queue::test_scope::ConfiguredTestScope::from_session(
        &root,
        &default_cfg,
        &paths,
        &session,
    );
    assert!(!unconfigured.is_match(&root, &root.join("smoke.test.mts")));

    let mut pw_cfg = crate::config::v2::NoMistakesConfig::default();
    pw_cfg.tests.playwright.test_include = vec!["e2e/**/*.mts".to_string()];
    pw_cfg.tests.playwright.test_exclude = vec!["e2e/helpers/**".to_string()];
    let pw_scope = crate::queue::test_scope::ConfiguredTestScope::from_session(
        &root, &pw_cfg, &paths, &session,
    );
    assert!(pw_scope.is_match(&root, &root.join("e2e/checkout.spec.mts")));
    assert!(!pw_scope.is_match(&root, &root.join("e2e/helpers/setup.mts")));
    assert!(!pw_scope.is_match(&root, &root.join("src/checkout.mts")));
}

#[test]
fn test_scope_files_still_participate_in_matching_for_all_three_main_parity_shapes() {
    let parity_root = |case: &str| {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/queue-ast-hop/test-scope-matching-parity")
            .join(case)
            .join("fixture")
    };

    let prod_producer =
        analyze_project(&parity_root("prod-producer-test-worker"), None, &[]).unwrap();
    assert!(
        prod_producer.check.is_empty(),
        "production producer matched by test-scope worker must stay silent: {:#?}",
        prod_producer.check
    );

    let prod_worker =
        analyze_project(&parity_root("prod-worker-test-producer"), None, &[]).unwrap();
    assert!(
        prod_worker.check.is_empty(),
        "production worker matched by test-scope producer must stay silent: {:#?}",
        prod_worker.check
    );

    let unresolved = analyze_project(
        &parity_root("prod-worker-test-unresolved-producer"),
        None,
        &[],
    )
    .unwrap();
    assert_eq!(unresolved.check.len(), 1, "{:#?}", unresolved.check);
    assert_eq!(
        unresolved.check[0].kind,
        "unmatched-worker-unresolved-producers"
    );
    assert!(
        unresolved.check[0]
            .message
            .contains("tests/gamma.test.mts:4"),
        "{:#?}",
        unresolved.check[0]
    );
}

#[test]
fn unresolved_worker_queue_name_abstains_from_unmatched_producer() {
    let abstain_root = |case: &str| {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/queue-ast-hop/unresolved-worker-abstain")
            .join(case)
            .join("fixture")
    };

    for (case, worker_file) in [
        ("dynamic-worker", "apps/workers/dynamic-worker.mts"),
        ("unresolved-reexport", "apps/workers/reexport-worker.mts"),
    ] {
        let report = analyze_project(&abstain_root(case), None, &[]).unwrap();
        assert!(
            report.check.is_empty(),
            "{case}: imported-constant producer must not report unmatched-producer when an unresolved-queue worker could match it: {:#?}",
            report.check
        );
        assert!(
            report.diagnostics.iter().any(|d| {
                d.file == worker_file && d.message == "dynamic or unresolved queue worker"
            }),
            "{case}: worker must still emit dynamic/unresolved diagnostic: {:#?}",
            report.diagnostics
        );
    }
}
