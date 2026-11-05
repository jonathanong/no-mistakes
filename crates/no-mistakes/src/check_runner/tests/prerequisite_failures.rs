use crate::check_runner::{graph_plan, prepared};
use no_mistakes::codebase::analysis_session::AnalysisSession;
use no_mistakes::codebase::check_outcome::{check_application_key, Prerequisite};
use std::path::PathBuf;

fn prepare_fixture(fixture: &str, config: &str) -> prepared::PreparedCheckInputs {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(fixture);
    prepared::prepare_with_session(
        &AnalysisSession::disabled(),
        &root,
        Some(&root.join(config)),
        None,
        None,
        false,
    )
    .unwrap()
}

#[test]
fn failed_playwright_preparation_marks_only_requested_consumers() {
    let mut inputs = prepare_fixture("check/partial-playwright", "valid.yml");
    let rule = inputs.config.rule_applications("playwright-coverage")[0].clone();
    for consumer in [
        "playwright-unique-test-ids",
        "playwright-prefer-test-id-locators",
    ] {
        let mut selected = rule.clone();
        selected.rule = consumer.into();
        inputs.config.rules.push(selected);
    }
    inputs.playwright = None;
    inputs.playwright_failure = Some("valid.config.ts: testDir could not be prepared".into());
    let session = AnalysisSession::new(None);
    prepared::failures::record(&session, &inputs);
    let failures = session.check_failures();
    assert_eq!(failures.len(), 3, "{failures:?}");
    assert!(failures
        .iter()
        .all(|failure| failure.consumer.starts_with("playwright-")
            && failure.message.contains("valid.config.ts: testDir")));
    assert!(!failures
        .iter()
        .any(|failure| failure.consumer == "no-empty-or-comments-only-files"));
}

#[test]
fn prepared_playwright_input_failures_keep_the_config_location() {
    let inputs = prepare_fixture("check/partial-playwright", ".no-mistakes.yml");
    assert!(inputs.playwright_failure.is_none());
    let session = AnalysisSession::new(None);
    prepared::failures::record(&session, &inputs);
    let failures = session.check_failures();
    assert!(
        failures
            .iter()
            .any(|failure| failure.consumer == "playwright-coverage"
                && failure
                    .file
                    .as_ref()
                    .is_some_and(|file| file.ends_with("playwright.config.ts"))
                && failure.property.as_deref() == Some("testDir")),
        "{failures:?}"
    );
}

#[test]
fn graph_preparation_failures_preserve_each_application_and_strict_errors() {
    let inputs = prepare_fixture(
        "playwright/partial-config-failure",
        "graph-scopes-check.yml",
    );
    let error = anyhow::anyhow!("playwright.projects.config.ts: testDir could not be prepared");
    let strict = AnalysisSession::disabled();
    let failure = graph_plan::failures::record(&strict, &inputs.config, &error).unwrap_err();
    assert!(failure
        .to_string()
        .contains("testDir could not be prepared"));
    assert!(strict.check_failures().is_empty());

    let session = AnalysisSession::new(None);
    session.enable_partial_checks();
    graph_plan::failures::initialize(&strict, &inputs.config);
    graph_plan::failures::initialize(&session, &inputs.config);
    for _ in 0..2 {
        graph_plan::failures::record(&session, &inputs.config, &error).unwrap();
    }
    graph_plan::failures::initialize(&session, &inputs.config);
    let applications = inputs.config.rule_applications("forbidden-dependencies");
    assert_eq!(applications.len(), 2);
    for (index, application) in applications.into_iter().enumerate() {
        let key = check_application_key(application, index + 1);
        let Some(Prerequisite::Failed(failures)) = session.check_prerequisite(&key) else {
            panic!("requested graph application must remain incomplete: {key}");
        };
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert_eq!(failures[0].consumer, "forbidden-dependencies");
    }
    let independent = inputs
        .config
        .rule_applications("no-empty-or-comments-only-files")[0];
    let key = check_application_key(independent, 1);
    assert!(matches!(
        session.check_prerequisite(&key),
        Some(Prerequisite::Ready(()))
    ));
}

#[test]
fn graph_preparation_merges_ready_facts_and_marks_only_failed_applications() {
    let inputs = prepare_fixture(
        "playwright/partial-config-failure",
        "graph-scopes-check.yml",
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    let session = AnalysisSession::new(None);
    session.enable_partial_checks();
    let requested = no_mistakes::codebase::rules::try_canonical_graph_plan(&inputs.config).unwrap();
    let mut playwright = Some(Default::default());
    let mut facts = Default::default();
    let graph = graph_plan::prepare(
        &root,
        &inputs.config,
        graph_plan::PreparedInputs {
            session: &session,
            codebase_config: &inputs.codebase_config,
            tsconfig: &inputs.tsconfig,
            visible_paths: &inputs.visible_paths,
            workflow_documents: None,
        },
        requested,
        &mut playwright,
        &mut facts,
    )
    .unwrap();
    assert!(graph.is_some());
    assert!(playwright.is_some());
    for (index, application) in inputs
        .config
        .rule_applications("forbidden-dependencies")
        .into_iter()
        .enumerate()
    {
        let outcome = session.check_prerequisite(&check_application_key(application, index + 1));
        if application
            .projects
            .iter()
            .any(|project| project == "broken")
        {
            assert!(matches!(outcome, Some(Prerequisite::Failed(_))));
        } else {
            assert!(matches!(outcome, Some(Prerequisite::Ready(()))));
        }
    }
    let mut facts = Default::default();
    let error = graph_plan::prepare(
        &root,
        &inputs.config,
        graph_plan::PreparedInputs {
            session: &AnalysisSession::disabled(),
            codebase_config: &inputs.codebase_config,
            tsconfig: &inputs.tsconfig,
            visible_paths: &inputs.visible_paths,
            workflow_documents: None,
        },
        requested,
        &mut None,
        &mut facts,
    )
    .err()
    .expect("strict preparation must preserve the config error");
    assert!(format!("{error:#}").contains("testDir"));
}
