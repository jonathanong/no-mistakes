use super::*;

#[test]
fn incomplete_boundary_sources_follow_actual_configured_route_selection() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check-consumer-isolation"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join("boundary.yml"))).unwrap();
    let files = [root.join("broken.ts"), root.join("routes/broken.ts")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.to_vec(),
        crate::codebase::check_facts::CheckFactPlan {
            server_route_client_boundary: true,
            ..Default::default()
        },
    );
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.enable_partial_checks();
    diagnostics_with_facts_for_aggregate_and_session(
        &root,
        &config,
        &facts,
        &sources,
        None,
        true,
        Some(&session),
    )
    .unwrap();
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(failures[0]
        .file
        .as_ref()
        .unwrap()
        .ends_with("routes/broken.ts"));
    let mut unrelated = config;
    unrelated.rules[0].include = vec!["broken.ts".to_string()];
    let session = session.scoped_checks();
    diagnostics_with_facts_for_aggregate_and_session(
        &root,
        &unrelated,
        &facts,
        &sources,
        None,
        true,
        Some(&session),
    )
    .unwrap();
    assert!(session.check_failures().is_empty());
}
