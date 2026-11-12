use super::*;
use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::config::v2::schema::StringOrList;
use std::path::PathBuf;

fn inputs() -> (PathBuf, PreparedCheckInputs) {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/check/partial-playwright"),
    );
    let prepared = super::super::super::prepared::prepare_with_session(
        &AnalysisSession::disabled(),
        &root,
        Some(&root.join("valid.yml")),
        None,
        None,
        false,
    )
    .unwrap();
    (root, prepared)
}

#[test]
fn setup_recovers_playwright_facts_without_a_prepared_rule_projection() {
    let (root, mut prepared) = inputs();
    assert!(prepared.playwright.take().is_some());
    let session = AnalysisSession::disabled();
    let plan = build(&session, &root, &prepared, false).unwrap();
    assert!(plan.playwright_fact_plan.is_some());
    assert!(plan.fact_demand.needs_shared_facts());
    assert!(plan.filesystem_rules_enabled);
    assert!(plan.prepared_graph.is_none());
    assert!(session.check_failures().is_empty());
}

#[test]
fn setup_preserves_strict_playwright_errors_and_retains_independent_partial_checks() {
    let (root, mut prepared) = inputs();
    prepared.playwright = None;
    prepared.config.tests.playwright.configs =
        Some(StringOrList::One("playwright.config.ts".into()));
    for partial in [false, true] {
        let session = AnalysisSession::disabled();
        if partial {
            session.enable_partial_checks();
        }
        let result = build(&session, &root, &prepared, false);
        if partial {
            let plan = result.unwrap();
            assert!(plan.playwright_fact_plan.is_none());
            assert!(plan.filesystem_rules_enabled);
            assert!(!plan.fact_demand.needs_shared_facts());
        } else {
            let error = result.err().expect("strict fact planning must fail");
            assert!(format!("{error:#}").contains("testDir"));
            assert!(session.check_failures().is_empty());
        }
    }
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
