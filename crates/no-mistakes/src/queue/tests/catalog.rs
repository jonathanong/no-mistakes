use super::*;

#[test]
fn prepared_catalog_keeps_symlinked_queue_producers_and_workers_in_one_namespace() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/tsconfig/symlink-workspace/link");
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible_paths);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            queue_factory_names: vec!["createQueue".to_string()],
            ..Default::default()
        },
    );
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible(&root, &[], &visible_paths);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());
    let report = analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &Default::default(),
        &[],
        &facts,
        &session,
    )
    .unwrap();

    assert!(
        report.producers.iter().any(|producer| {
            producer.file.ends_with("/src/producer.ts")
                && producer
                    .queue_file
                    .as_deref()
                    .is_some_and(|path| path.ends_with("/src/queues.ts"))
                && producer.queue_name.as_deref() == Some("emails")
        }),
        "{report:#?}"
    );
    assert!(
        report.workers.iter().any(|worker| {
            worker.file.ends_with("/src/worker.ts")
                && worker
                    .queue_file
                    .as_deref()
                    .is_some_and(|path| path.ends_with("/src/queues.ts"))
                // An inline namespace dispatch is the processor, not the imported helper file.
                && worker.processor_file.is_none()
        }),
        "{report:#?}"
    );
}

#[test]
fn prepared_catalog_indexed_queue_report_matches_plain_report_for_package_aliases() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/queue-tsconfig-catalog");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(&root);
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible_paths);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            ..Default::default()
        },
    );
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::from_visible(&root, &[], &visible_paths);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(crate::diagnostics::current());

    let plain = analyze_project_with_prepared_facts_and_catalog_and_session(
        &root,
        &catalog,
        &Default::default(),
        &[],
        &facts,
        &session,
    )
    .unwrap();
    let indexed = analyze_project_with_prepared_facts_indexed_and_catalog_and_session(
        &root,
        &catalog,
        &Default::default(),
        &[],
        &facts,
        &session,
    )
    .unwrap();

    let mut plain_producers = plain.producers.clone();
    let mut indexed_producers = indexed.report().producers.clone();
    plain_producers.sort();
    indexed_producers.sort();
    assert_eq!(indexed_producers, plain_producers);
    let mut plain_workers = plain.workers.clone();
    let mut indexed_workers = indexed.report().workers.clone();
    plain_workers.sort();
    indexed_workers.sort();
    assert_eq!(indexed_workers, plain_workers);
    assert_eq!(indexed.report().jobs, plain.jobs);
    assert_eq!(indexed.report().edges, plain.edges);
    assert_eq!(indexed.report().diagnostics, plain.diagnostics);
    assert_eq!(indexed.report().check, plain.check);
    assert!(plain.check.is_empty(), "{plain:#?}");
    for (package, queue) in [("a", "a-queue"), ("b", "b-queue")] {
        let file = format!("packages/{package}/src/enqueue.ts");
        let queue_file = format!("packages/{package}/src/queues/email.ts");
        assert!(
            plain.producers.iter().any(|producer| {
                producer.file == file
                    && producer.queue_file.as_deref() == Some(&queue_file)
                    && producer.queue_name.as_deref() == Some(queue)
            }),
            "{plain:#?}"
        );
    }
}
