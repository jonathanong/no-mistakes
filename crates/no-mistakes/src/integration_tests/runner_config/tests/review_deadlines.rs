use super::*;
use crate::integration_tests::types::{ConfigDeadlineEvidence, RunnerDeadlineStatus};

fn evidence(
    vitest: &[&str],
    playwright: &[&str],
    files: &[&str],
) -> Vec<crate::integration_tests::types::RunnerConfigDeadlineEvidence> {
    let root = fixture_root("declared-deadlines");
    let paths: Vec<_> = files.iter().map(|file| root.join(file)).collect();
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&paths),
    )));
    let catalog = Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &paths,
    ));
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::Many(
        vitest.iter().map(|v| (*v).into()).collect(),
    ));
    config.tests.playwright.configs = Some(StringOrList::Many(
        playwright.iter().map(|v| (*v).into()).collect(),
    ));
    let prepared = prepare_runner_configs_with_deadline_evidence(
        &root,
        &config,
        &paths,
        catalog,
        Arc::clone(&sources),
    );
    crate::ast::begin_parse_count_this_thread(&root);
    let result = crate::ast::with_request_parse_cache(|| {
        let parsed = prepared.parse_all().unwrap();
        for file in parsed.files.values() {
            for result in &file.results {
                if result.deadline_error.is_some() {
                    assert!(result.projects.as_ref().unwrap().is_empty());
                }
            }
        }
        let facts = crate::codebase::check_facts::CheckFactMap {
            integration_runner_configs: parsed.files,
            ..Default::default()
        };
        let evidence = prepared.deadline_evidence(&facts);
        assert_eq!(prepared.deadline_evidence(&facts), evidence);
        evidence
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(sources.physical_read_count(), paths.len());
    for path in paths {
        assert_eq!(counts.get(&path), Some(&1));
    }
    result
}

#[test]
fn review_deadlines_opaque_roots_fail_but_literal_empty_workspaces_remain_prepared() {
    let opaque = [
        "roots/functional/vitest.config.ts",
        "roots/async/vitest.config.ts",
        "roots/opaque-workspace/vitest.workspace.ts",
    ];
    let result = evidence(&opaque, &[], &opaque);
    let vitest = result
        .iter()
        .find(|runner| runner.framework == "vitest")
        .unwrap();
    assert_eq!(vitest.status, RunnerDeadlineStatus::Failed);
    assert_eq!(vitest.configs.len(), 3);
    assert!(vitest.configs.iter().all(|config| matches!(config, ConfigDeadlineEvidence::Failed { error, .. } if error.contains("unsupported opaque"))));
    let empty = [
        "roots/empty/vitest.workspace.ts",
        "roots/wrapped-empty/vitest.workspace.ts",
        "roots/namespace-empty/vitest.workspace.ts",
    ];
    let result = evidence(&empty, &[], &empty);
    let vitest = result
        .iter()
        .find(|runner| runner.framework == "vitest")
        .unwrap();
    assert_eq!(vitest.status, RunnerDeadlineStatus::Prepared);
    assert_eq!(vitest.configs.len(), 3);
    assert!(vitest.configs.iter().all(|config| matches!(config, ConfigDeadlineEvidence::Prepared { projects, .. } if projects.is_empty())));
}

#[test]
fn review_deadlines_explicit_globs_use_visible_inventory_and_ignore_nonmatches() {
    let files = [
        "roots/static/vitest.config.ts",
        "roots/functional/vitest.config.ts",
    ];
    let result = evidence(
        &[
            "roots/*/vitest.config.ts",
            "roots/static/vitest.config.ts",
            "absent/*.ts",
            "[unclosed",
        ],
        &[],
        &files,
    );
    let vitest = result
        .iter()
        .find(|runner| runner.framework == "vitest")
        .unwrap();
    assert_eq!(vitest.configs.len(), 2);
    assert!(vitest.configs.iter().any(|config| matches!(config, ConfigDeadlineEvidence::Prepared { config, projects } if config == "roots/static/vitest.config.ts" && projects.len() == 1)));
    assert!(vitest.configs.iter().any(|config| matches!(config, ConfigDeadlineEvidence::Failed { config, .. } if config == "roots/functional/vitest.config.ts")));
    let result = evidence(&["absent/*.ts", "[unclosed"], &[], &[]);
    let vitest = result
        .iter()
        .find(|runner| runner.framework == "vitest")
        .unwrap();
    assert_eq!(vitest.status, RunnerDeadlineStatus::Prepared);
    assert!(vitest.configs.is_empty());
}

#[test]
fn review_deadlines_missing_literal_is_a_retained_failure_not_an_empty_glob() {
    let root = fixture_root("declared-deadlines");
    let paths = Vec::new();
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&paths),
    )));
    let catalog = Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &paths,
    ));
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("missing-literal.ts".into()));
    let prepared = prepare_runner_configs_with_deadline_evidence(
        &root,
        &config,
        &paths,
        catalog,
        Arc::clone(&sources),
    );
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let path = root.join("missing-literal.ts");
    let file = prepared
        .parse_path_for_facts_with_session(&session, &path)
        .unwrap();
    assert!(file.results[0].projects.is_err());
    let facts = crate::codebase::check_facts::CheckFactMap {
        integration_runner_configs: [(path, file)].into_iter().collect(),
        ..Default::default()
    };
    let evidence = prepared.deadline_evidence(&facts);
    assert_eq!(evidence[1].status, RunnerDeadlineStatus::Failed);
    assert!(
        matches!(&evidence[1].configs[0], ConfigDeadlineEvidence::Failed { config, .. } if config == "missing-literal.ts")
    );
    assert_eq!(prepared.deadline_evidence(&facts), evidence);
    assert_eq!(sources.physical_read_count(), 1);
}
