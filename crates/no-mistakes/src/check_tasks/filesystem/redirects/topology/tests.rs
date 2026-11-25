use super::*;
use no_mistakes::codebase::{analysis_session::AnalysisSession, ts_source::VisiblePathSnapshot};

#[test]
fn missing_prepared_topology_keeps_its_consumer_in_partial_mode() {
    let saved = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/workflow-topology-policy");
    let root = saved.join("fixture/merge-location-unavailable");
    let config = no_mistakes::config::v2::load_v2_config(
        &root,
        Some(&saved.join("configs/merge-location-unavailable.yml")),
    )
    .unwrap();
    let snapshot = VisiblePathSnapshot::from_paths(&root, &[]);
    let sources = snapshot.source_store_for(&root);
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    for analysis in [None, Some(session.as_ref())] {
        let prepared = rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
            session: analysis,
            next_config_resolver: None,
            snapshot: &snapshot,
            vitest_catalog: None,
            sources: std::sync::Arc::clone(&sources),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        };
        let mut diagnostics = AnalysisDiagnostics::default();
        let result = run(&root, &config, &prepared, true, &mut diagnostics);
        if analysis.is_none() {
            assert!(result
                .expect_err("missing topology must fail")
                .to_string()
                .contains("prepared workflow documents"));
        } else {
            assert!(!result
                .unwrap()
                .rule_configured(rules::WORKFLOW_TOPOLOGY_POLICY));
        }
        assert!(diagnostics.findings.is_empty());
    }
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].consumer, rules::WORKFLOW_TOPOLOGY_POLICY);
}
