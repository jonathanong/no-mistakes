use super::*;
use no_mistakes::codebase::{analysis_session::AnalysisSession, ts_source::VisiblePathSnapshot};

#[test]
fn missing_prepared_policy_never_reads_or_parses_sources() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/vitest-project-policy/fixture/missing");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    let config =
        no_mistakes::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    let files = vec![root.join("vitest.config.ts")];
    let snapshot = VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let session = AnalysisSession::disabled();
    #[cfg(feature = "test-instrumentation")]
    no_mistakes::ast::begin_parse_count(&root);
    for session in [None, Some(session.as_ref())] {
        let prepared = rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
            session,
            next_config_resolver: None,
            snapshot: &snapshot,
            vitest_catalog: None,
            sources: std::sync::Arc::clone(&sources),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        };
        let error = run(&root, &config, true, &files, prepared, None, false)
            .err()
            .expect("missing prepared diagnostics must fail");
        assert!(error
            .to_string()
            .contains("requires prepared policy diagnostics"));
        assert_eq!(sources.physical_read_count(), 0);
    }
    #[cfg(feature = "test-instrumentation")]
    assert!(no_mistakes::ast::finish_parse_count(&root).is_empty());
}
