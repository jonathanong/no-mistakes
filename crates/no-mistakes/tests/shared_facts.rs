use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use no_mistakes::codebase::ts_source::discover_files;
use no_mistakes::codebase::unique_exports;
use no_mistakes::queue;
use std::path::PathBuf;

fn codebase_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis")
        .join(name)
        .join("fixture")
}

fn queue_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/queue-ast-hop")
        .join(name)
        .join("fixture")
}

#[test]
fn unique_exports_public_api_uses_shared_facts() {
    let root = codebase_fixture("unique-exports-basic");

    let findings = unique_exports::analyze_project(&root, None, None).unwrap();

    assert_eq!(findings.len(), 2);
}

#[test]
fn queue_public_api_uses_shared_facts() {
    let root = queue_fixture("basic");
    let facts = collect_check_facts(
        &root,
        discover_files(&root, &[]),
        CheckFactPlan {
            queue: true,
            ..Default::default()
        },
    );

    let report = queue::analyze_project_with_facts(&root, None, &[], &facts).unwrap();

    assert_eq!(report.check, vec![]);
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "enqueue.ts" && edge.to == "queues.ts#sendWelcome"));
}

#[test]
fn public_session_and_integration_apis_reuse_canonical_config_and_runner_facts() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/parser-count/playwright");
    let session = AnalysisSession::disabled();
    let snapshot = std::sync::Arc::new(no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(
        &root,
    ));

    session.insert_visible_paths(&root, std::sync::Arc::clone(&snapshot));
    assert!(std::sync::Arc::ptr_eq(
        &session.visible_paths(&root),
        &snapshot
    ));
    let config = session
        .config(&root, None)
        .expect("fixture config should load through the session source store");
    assert_eq!(
        config.tests.playwright.configs.as_ref().unwrap().values(),
        ["playwright.config.ts"]
    );

    let integration_root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests/basic/fixture"),
    );
    let findings = no_mistakes::integration_tests::check(&integration_root, None)
        .expect("runner configuration should be reusable by the public integration API");
    assert_eq!(findings.len(), 6);
}
