use super::*;

#[test]
fn partial_integration_keeps_independent_tests_and_ignores_unused_broken_sources() {
    let root = fixture("basic");
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible = snapshot.paths_for(&root);
    let config = crate::config::v2::load_v2_config_from_visible(&root, None, &visible).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible).unwrap();
    let runner_configs = runner_config::prepare(&root, &config, &visible, &tsconfig);
    let files = crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible);
    let mut shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            integration: true,
            integration_runner_configs: Some(std::sync::Arc::new(runner_configs)),
            ..Default::default()
        },
    );
    let expected =
        check_with_prepared_facts(&root, &config, &shared, &tsconfig, &snapshot).unwrap();
    let unrelated = fixture("parse-errors").join("src/syntax-error.ts");
    shared.ts.insert(
        unrelated,
        crate::codebase::check_facts::CheckFileFacts {
            parse_error: Some("unrelated parse failure".into()),
            ..Default::default()
        }
        .into(),
    );
    assert_eq!(
        check_with_prepared_facts(&root, &config, &shared, &tsconfig, &snapshot).unwrap(),
        expected
    );
    let helper = root.join("helpers/openai.mts");
    shared.ts.insert(
        helper.clone(),
        crate::codebase::check_facts::CheckFileFacts {
            parse_error: Some("used helper parse failure".into()),
            ..Default::default()
        }
        .into(),
    );
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.enable_partial_checks();
    let findings = check_with_prepared_facts_and_session(
        &root, &config, &shared, &tsconfig, &snapshot, &session,
    )
    .unwrap();
    assert!(findings
        .iter()
        .any(|finding| finding.test_name.as_deref() == Some("direct integration in unit suite")));
    assert!(!findings
        .iter()
        .any(|finding| finding.test_name.as_deref() == Some("helper integration in unit suite")));
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].file.as_ref(), Some(&helper));
}
