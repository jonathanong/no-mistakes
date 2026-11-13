use super::*;
use no_mistakes::codebase::analysis_session::AnalysisSession;
use std::path::PathBuf;

#[test]
fn setup_retains_canonical_playwright_rule_facts_and_prerequisite_failures() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/partial-playwright"),
    );
    for (config, failed) in [("valid.yml", false), (".no-mistakes.yml", true)] {
        let session = AnalysisSession::disabled();
        session.enable_partial_checks();
        let prepared = super::super::super::prepared::prepare_with_session(
            &session,
            &root,
            Some(&root.join(config)),
            None,
            None,
            false,
        )
        .unwrap();
        assert!(prepared.playwright.is_some());
        assert!(prepared.playwright_failure.is_none());
        let plan = build(&session, &root, &prepared, false).unwrap();
        assert!(plan.playwright_fact_plan.is_some());
        assert!(plan.fact_demand.needs_shared_facts());
        assert!(plan.filesystem_rules_enabled);
        assert!(plan.prepared_graph.is_none());
        let failures = session.check_failures();
        assert_eq!(!failures.is_empty(), failed, "{config}: {failures:?}");
        assert!(failures
            .iter()
            .all(|failure| failure.consumer == "playwright-coverage"
                && failure.property.as_deref() == Some("testDir")));
    }
}

#[test]
fn setup_derives_graph_only_playwright_facts_without_a_rule_projection() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/partial-config-failure"),
    );
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    let prepared = super::super::super::prepared::prepare_with_session(
        &session,
        &root,
        Some(&root.join("graph-scopes-check.yml")),
        None,
        None,
        false,
    )
    .unwrap();
    assert!(prepared.playwright.is_none());
    assert!(prepared.playwright_failure.is_none());
    let plan = build(&session, &root, &prepared, false).unwrap();
    assert!(plan.prepared_graph.is_some());
    assert!(plan.playwright_fact_plan.is_some());
    assert!(plan.fact_demand.needs_shared_facts());
    assert!(plan.plan.graph.imports);
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].consumer, "forbidden-dependencies");
    assert_eq!(failures[0].property.as_deref(), Some("testDir"));
}

#[test]
fn setup_does_not_retry_a_failed_playwright_prerequisite() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check-runner/unresolved-playwright-root");
    let session = AnalysisSession::disabled();
    let prepared = super::super::super::prepared::prepare_with_session(
        &session, &root, None, None, None, false,
    )
    .unwrap();
    assert!(prepared.playwright.is_none());
    assert!(prepared
        .playwright_failure
        .as_ref()
        .is_some_and(|message| message.contains("cannot infer the Next.js app root")));
    let plan = build(&session, &root, &prepared, false).unwrap();
    assert!(plan.playwright_fact_plan.is_none());
    assert!(!plan.fact_demand.needs_shared_facts());
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].consumer, "playwright-coverage");
}

#[test]
fn empty_analysis_keeps_requested_report_evidence() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check-runner/empty");
    let report =
        super::super::run_all_with_sql_verdicts(root, None, None, true, true, true).unwrap();
    assert!(report.rules.is_empty());
    assert!(report.incomplete.is_empty());
    let evidence = report.runner_config_deadlines.as_ref().unwrap();
    assert_eq!(evidence.len(), 2);
    assert!(evidence.iter().all(|runner| runner.configs.is_empty()));
    let json = crate::check_runner::results::json_value(&report);
    assert_eq!(
        json["runnerConfigDeadlines"],
        serde_json::json!([
            {"framework": "playwright", "status": "notRequested", "configs": []},
            {"framework": "vitest", "status": "notRequested", "configs": []},
        ])
    );
    for key in ["sqlVerdicts", "suppressed"] {
        assert_eq!(json[key], serde_json::json!([]), "{key}");
    }
}
