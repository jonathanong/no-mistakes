use super::*;
use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/integration-tests")
            .join(name)
            .join("fixture"),
    )
}

fn integration_policy() -> TestProjectPolicy {
    TestProjectPolicy {
        integration_suites: BTreeMap::from([("integration".to_string(), Vec::new())]),
        ..Default::default()
    }
}

fn prepare_vitest(root: &Path, config_path: StringOrList) -> PreparedIntegrationRunnerConfigs {
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(config_path);
    config
        .tests
        .vitest
        .projects
        .insert("unit".to_string(), integration_policy());
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)
            .unwrap();
    prepare(root, &config, &visible_paths, &tsconfig)
}

#[test]
fn prepared_runner_records_cached_parse_errors_as_project_results() {
    let root = fixture_root("parse-errors");
    let prepared = prepare_vitest(
        &root,
        StringOrList::One("vitest.syntax-error.mts".to_string()),
    );
    let path = root.join("vitest.syntax-error.mts");

    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = prepared
        .parse_path_for_facts_with_session(&session, &path)
        .unwrap();
    assert_eq!(facts.results.len(), 1);
    assert!(facts.results[0]
        .projects
        .as_ref()
        .unwrap_err()
        .contains("failed to parse"));
    assert!(prepared
        .parse_error(&root.join("unprepared.mts"), "ignored".to_string())
        .is_none());
}

#[test]
fn prepared_runner_records_config_read_errors_as_project_results() {
    let root = fixture_root("parse-errors");
    let prepared = prepare_vitest(&root, StringOrList::One("src".to_string()));

    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = prepared
        .parse_path_for_facts_with_session(&session, &root.join("src"))
        .unwrap();
    assert_eq!(facts.results.len(), 1);
    assert!(facts.results[0].projects.is_err());
}

#[test]
fn prepared_runner_deduplicates_config_parsing_and_supports_direct_fact_parsing() {
    let root = fixture_root("basic");
    let raw = "vitest.config.mts".to_string();
    let prepared = prepare_vitest(&root, StringOrList::Many(vec![raw.clone(), raw.clone()]));
    let parsed = prepared.parse_all().unwrap();
    assert_eq!(parsed.files.len(), 1);

    let path = root.join(raw);
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, source| {
        prepared.parse_program(&path, program, source)
    })
    .unwrap()
    .unwrap();
    assert_eq!(facts.results.len(), 2);
    assert!(facts.results.iter().all(|result| result.projects.is_ok()));
}

#[test]
fn prepared_runner_uses_session_source_and_parser_gateways_once() {
    let root = fixture_root("basic");
    let path = root.join("vitest.config.mts");
    let prepared = prepare_vitest(&root, StringOrList::One("vitest.config.mts".to_string()));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));

    crate::ast::with_request_parse_cache(|| {
        for _ in 0..2 {
            assert!(prepared
                .parse_path_for_facts_with_session(&session, &path)
                .is_some());
        }
    });

    let snapshot = session.work_snapshot();
    assert_eq!(snapshot.source_reads[&path], 1);
    assert_eq!(snapshot.parse_attempts[&path], 1);
    let work = observer.snapshot().work;
    assert_eq!(work["source.requests"], 2);
    assert_eq!(work["source.reads"], 1);
    assert_eq!(work["source.cache_hits"], 1);
    assert_eq!(work["parse.requests"], 2);
    assert_eq!(work["parse.files"], 1);
}

#[test]
fn runner_config_request_cache_does_not_recollect_prepared_config_facts() {
    let root = fixture_root("basic");
    let path = root.join("vitest.config.mts");
    let prepared = prepare_vitest(&root, StringOrList::One("vitest.config.mts".to_string()));
    let primary_plan = crate::codebase::check_facts::CheckFactPlan {
        integration_runner_configs: Some(Arc::new(prepared.clone())),
        ..Default::default()
    };
    let fact_plan = RunnerConfigFactPlan {
        root: root.clone(),
        primary_files: [path.clone()].into(),
        graph_files: Default::default(),
        primary_plan,
        graph_plan: Default::default(),
        playwright: None,
    };
    let source = std::fs::read_to_string(&path).unwrap();

    let (_, helper_facts) = prepared.with_request_cache(Some(fact_plan), || {
        with_program(&path, &source, |_, _| ()).unwrap()
    });

    assert!(helper_facts.is_empty());
}

#[test]
fn parsed_runner_configs_filter_analyses_and_return_matching_projects() {
    let root = PathBuf::from("fixture");
    let config_path = root.join("vitest.config.ts");
    let retained_analysis = root.join("helpers/retained.ts");
    let omitted_analysis = root.join("helpers/omitted.ts");
    let project = ConfigProject {
        config: Some("vitest.config.ts".to_string()),
        workspace: false,
        policy_name: Some("unit".to_string()),
        runner_project_arg: Some("unit".to_string()),
        scope: Some("tests".to_string()),
        include: vec!["tests/**/*.test.ts".to_string()],
        exclude: Vec::new(),
        vitest_setup: Vec::new(),
    };
    let parsed = ParsedRunnerConfigs::with_files(BTreeMap::from([(
        config_path.clone(),
        RunnerConfigFileFacts {
            results: vec![ProjectResult {
                framework: Framework::Vitest,
                raw: "vitest.config.ts".to_string(),
                projects: Ok(vec![project.clone()]),
            }],
            analyses: BTreeMap::from([
                (retained_analysis.clone(), FileAnalysis::default()),
                (omitted_analysis, FileAnalysis::default()),
            ]),
        },
    )]));
    let tsconfig_catalog = Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
        &root,
        crate::codebase::ts_resolver::TsConfig {
            dir: root.clone(),
            paths: Vec::new(),
            paths_dir: root.clone(),
            base_url: None,
        },
        None,
    ));
    let plan = PreparedIntegrationRunnerConfigs {
        root: root.clone(),
        specs: vec![RunnerConfigSpec {
            framework: Framework::Vitest,
            raw: "vitest.config.ts".to_string(),
            path: config_path,
        }],
        tsconfig_catalog,
        visible_files: [root.join("vitest.config.ts")].into(),
        sources: None,
    };

    let analyses = parsed.analyses_for(std::slice::from_ref(&retained_analysis));
    assert_eq!(analyses.len(), 1);
    assert!(analyses.contains_key(&retained_analysis));
    assert_eq!(
        parsed.projects_for(&plan, Framework::Vitest).unwrap().len(),
        1
    );
    assert!(parsed.covers(&plan));

    let missing_path = root.join("missing.vitest.config.ts");
    let missing_plan = PreparedIntegrationRunnerConfigs {
        specs: vec![RunnerConfigSpec {
            framework: Framework::Vitest,
            raw: "missing.vitest.config.ts".to_string(),
            path: missing_path.clone(),
        }],
        visible_files: [missing_path].into(),
        ..plan
    };
    assert!(parsed
        .projects_for(&missing_plan, Framework::Vitest)
        .unwrap_err()
        .to_string()
        .contains("missing prepared vitest config"));
}
