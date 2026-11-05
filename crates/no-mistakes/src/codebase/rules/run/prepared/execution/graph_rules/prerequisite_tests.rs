use super::*;
use crate::codebase::check_outcome::{check_application_key, CheckFailure, Prerequisite};

#[test]
fn failed_named_graph_prerequisite_skips_only_its_original_application() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/partial-config-failure"),
    );
    let mut config = crate::config::v2::load_v2_config(
        &root,
        Some(&root.join("reachability-prerequisites.yml")),
    )
    .unwrap();
    config.rules[0].resolved_application_index = Some(7);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph =
        DepGraph::build_with_plan(&root, &tsconfig, GraphBuildPlan::imports_and_workspace())
            .unwrap();
    let shared = crate::codebase::check_facts::CheckFactMap {
        files: vec![
            root.join("good/page.tsx"),
            root.join("broken/page.tsx"),
            root.join("tests/good.spec.ts"),
        ],
        ..Default::default()
    };
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    session.enable_partial_checks();
    session.set_check_prerequisite("codebase.canonical-graph", Prerequisite::Ready(()));
    session.set_check_prerequisite(
        &check_application_key(&config.rules[0], 1),
        Prerequisite::Failed(vec![CheckFailure {
            consumer: REQUIRED_ENTRYPOINT_REACHABILITY.into(),
            file: Some(root.join("playwright.projects.config.ts")),
            property: Some("testDir".into()),
            message: "failed app graph projection".into(),
        }]),
    );
    let output = graph_rule_findings_with_session(
        GraphRuleRequest {
            root: &root,
            config: &config,
            config_path: None,
            shared: &shared,
            prepared_graph: None,
            dependency_graph: Some(&graph),
            inferred_roots: None,
            sources: None,
            tsconfig_catalog: None,
            prepared_vitest_projects: None,
            prepared_playwright_projects: None,
        },
        Some(&session),
    )
    .unwrap();
    assert_eq!(output.findings.len(), 1, "{:?}", output.findings);
    assert!(output.findings[0].file.ends_with("good/page.tsx"));
    assert!(output.findings[0].message.contains("not runtime-reachable"));
    assert_eq!(session.check_failures().len(), 1);
    assert!(matches!(
        session.check_prerequisite("codebase.canonical-graph"),
        Some(Prerequisite::Ready(()))
    ));
}
