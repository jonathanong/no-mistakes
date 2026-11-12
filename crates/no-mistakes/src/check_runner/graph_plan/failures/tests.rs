use super::*;
use no_mistakes::codebase::dependencies::graph::{
    prepare_graph_config_with_test_filter_partial, PlaywrightPreparationFailure,
};
use no_mistakes::codebase::test_filter::TestFileFilter;
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use std::path::PathBuf;

fn fixture_failure() -> (
    PathBuf,
    NoMistakesConfig,
    VisiblePathSnapshot,
    PlaywrightPreparationFailure,
) {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/partial-config-failure"),
    );
    let config =
        no_mistakes::config::v2::load_v2_config(&root, Some(&root.join("graph-scopes-check.yml")))
            .unwrap();
    let visible = VisiblePathSnapshot::new(&root);
    let requested = no_mistakes::codebase::rules::try_canonical_graph_plan(&config)
        .unwrap()
        .unwrap();
    let mut graph = prepare_graph_config_with_test_filter_partial(
        &root,
        requested,
        &Default::default(),
        &config,
        &visible,
        TestFileFilter::new(&root, &config),
    )
    .unwrap();
    let (_, failures) = graph.playwright_fact_plan_partial(&root, &Default::default(), &visible);
    let broken_rule = config
        .rules
        .iter()
        .find(|rule| rule.projects.iter().any(|project| project == "broken"))
        .unwrap();
    let failure = failures
        .into_iter()
        .find(|failure| {
            failure.project.as_deref() == Some("broken")
                && failure.file.as_deref()
                    == Some(root.join("playwright.projects.config.ts").as_path())
                && failure
                    .affects_rule(&root, &config, broken_rule, &visible.paths_for(&root))
                    .unwrap()
        })
        .expect("saved malformed project must produce a failure in the broken app scope");
    (root, config, visible, failure)
}

#[test]
fn prepared_failure_keeps_non_test_directory_error_location() {
    let (root, config, visible, mut failure) = fixture_failure();
    failure.message = "Playwright configuration could not be read".into();
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    initialize(&session, &config);
    record_prepared(
        &root,
        &session,
        &config,
        &visible.paths_for(&root),
        &failure,
    )
    .unwrap();
    let failures = session.check_failures();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].file, failure.file);
    assert_eq!(failures[0].message, failure.message);
    assert_eq!(failures[0].property, None);
}

#[test]
fn failure_attribution_preserves_invalid_rule_demand_and_filter_errors() {
    let (root, config, visible, failure) = fixture_failure();
    for invalid_filter in [false, true] {
        let mut invalid = config.clone();
        let rule = invalid
            .rules
            .iter_mut()
            .find(|rule| rule.projects.iter().any(|project| project == "broken"))
            .unwrap();
        if invalid_filter {
            rule.include = vec!["[".into()];
        } else {
            rule.options = serde_yaml::from_str("{ relationships: false }").unwrap();
        }
        let session = AnalysisSession::disabled();
        session.enable_partial_checks();
        let error = record_prepared(
            &root,
            &session,
            &invalid,
            &visible.paths_for(&root),
            &failure,
        )
        .unwrap_err();
        assert!(session.check_failures().is_empty());
        let message = format!("{error:#}");
        if invalid_filter {
            assert!(message.contains("glob"), "{message}");
        } else {
            assert!(message.contains("relationships"), "{message}");
            let error = record(&session, &invalid, &anyhow::anyhow!("source failure")).unwrap_err();
            assert!(format!("{error:#}").contains("relationships"));
            let failures = session.check_failures();
            assert_eq!(failures.len(), 1);
            assert_eq!(failures[0].message, "source failure");
        }
    }
}
