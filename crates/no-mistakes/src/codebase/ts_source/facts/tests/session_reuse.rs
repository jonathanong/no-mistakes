#[test]
fn session_fact_collection_reads_through_the_prepared_dataset_store() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/analysis-dataset/source-store");
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let alpha = root.join("alpha.ts");
    let beta = root.join("beta.ts");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session = crate::codebase::analysis_session::AnalysisSession::new(Some(
        std::sync::Arc::clone(&observer),
    ));
    let _snapshot = session.visible_paths(&root);
    let sources = session
        .existing_sources_for(&root)
        .expect("visible_paths prepares a dataset source store");
    assert_eq!(sources.inventory().metadata_stat_count(), 0);
    assert_eq!(sources.physical_read_count(), 0);

    let facts = collect_ts_facts_with_session_and_context(
        &session,
        &[alpha.clone(), beta.clone()],
        TsFactPlan::imports(),
        &TsFactContext::new(&root),
    );

    assert!(facts.contains_key(&alpha));
    assert!(facts.contains_key(&beta));
    assert_eq!(
        sources.physical_read_count(),
        2,
        "session collection must read through the dataset store, not a fresh FileInventory"
    );
    assert_eq!(
        sources.inventory().metadata_stat_count(),
        0,
        "prepared discovery classifications must not be restated"
    );

    let _again = collect_ts_facts_with_session_and_context(
        &session,
        &[alpha, beta],
        TsFactPlan::imports(),
        &TsFactContext::new(&root),
    );
    assert_eq!(sources.physical_read_count(), 2);
}

#[test]
fn session_fact_collection_without_discovery_still_reads_the_file_list() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/analysis-dataset/source-store");
    let alpha = crate::codebase::ts_resolver::normalize_path(&root.join("alpha.ts"));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    assert!(session.existing_sources_for(&root).is_none());

    let facts = collect_ts_facts_with_session_and_context(
        &session,
        std::slice::from_ref(&alpha),
        TsFactPlan::imports(),
        &TsFactContext::new(&root),
    );
    assert!(facts.contains_key(&alpha));
}
