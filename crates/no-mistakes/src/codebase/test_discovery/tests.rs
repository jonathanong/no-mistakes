use super::*;
use crate::config::v2::schema::{StringOrList, TestProjectPolicy};
use std::collections::BTreeMap;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis")
            .join(name)
            .join("fixture"),
    )
}

fn runner_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Result<Vec<ConfigProject>> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    let tsconfig = resolve_tsconfig_lossy(root, &visible_paths);
    projects::runner_projects_from_visible(root, config, runner, &visible_paths, &tsconfig)
}

fn runner_projects_lossy(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Vec<ConfigProject> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    let tsconfig = resolve_tsconfig_lossy(root, &visible_paths);
    projects::runner_projects_lossy_from_visible(root, config, runner, &visible_paths, &tsconfig)
}

fn discover_from_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
    projects: Vec<ConfigProject>,
) -> Result<DiscoveredTests> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    let tsconfig = resolve_tsconfig_lossy(root, &visible_paths);
    discover_from_projects_from_visible(
        root,
        config,
        runner,
        projects,
        None,
        &visible_paths,
        &tsconfig,
    )
}

mod runner_basics;
use runner_basics::prepare_test_projects_from_visible;

#[test]
fn vitest_explicit_project_matches_playwright_owned_file() {
    let root = fixture_root("symbols-output");
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["src/utils.mts".to_string()],
            ..Default::default()
        },
    );
    let projects = vec![ConfigProject {
        config: Some("vitest.config.mts".to_string()),
        policy_name: Some("browser".to_string()),
        runner_project_arg: Some("browser".to_string()),
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
    }];

    let discovered = discover_from_projects(&root, &config, TestRunner::Vitest, projects).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert_eq!(rel_tests, vec!["src/utils.mts"]);
}

#[test]
fn target_metadata_uses_executable_project_name_only() {
    let root = fixture_root("symbols-output");
    let config = NoMistakesConfig::default();
    let projects = vec![ConfigProject {
        config: Some("playwright.config.ts".to_string()),
        policy_name: Some("top-level-config-name".to_string()),
        runner_project_arg: None,
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
    }];

    let discovered =
        discover_from_projects(&root, &config, TestRunner::Playwright, projects).unwrap();
    let target = discovered
        .targets_by_path
        .values()
        .next()
        .unwrap()
        .first()
        .unwrap();

    assert_eq!(target.project, None);
    assert!(!target.runner_args.contains(&"--project".to_string()));
}

#[test]
fn policy_only_project_inherits_single_explicit_runner_config() {
    let root = fixture_root("symbols-output");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One(
        "configs/vitest.workspace.mts".to_string(),
    ));
    config.tests.vitest.projects.insert(
        "dynamic".to_string(),
        TestProjectPolicy {
            include: vec!["src/utils.mts".to_string()],
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let projects = runner_projects_lossy(&root, &config, TestRunner::Vitest);

    let project = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("dynamic"))
        .unwrap();
    assert_eq!(
        project.config.as_deref(),
        Some("configs/vitest.workspace.mts")
    );
}

#[test]
fn policy_only_project_does_not_guess_from_multiple_explicit_configs() {
    let root = fixture_root("symbols-output");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::Many(vec![
        "configs/vitest.workspace.mts".to_string(),
        "configs/vitest.browser.mts".to_string(),
    ]));
    config.tests.vitest.projects.insert(
        "dynamic".to_string(),
        TestProjectPolicy {
            include: vec!["src/utils.mts".to_string()],
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let projects = runner_projects_lossy(&root, &config, TestRunner::Vitest);

    let project = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("dynamic"))
        .unwrap();
    assert_eq!(project.config, None);
}

#[test]
fn policy_only_project_discovery_preserves_fallback_tests_outside_policy() {
    let root = fixture_root("test-discovery-policy-fallback");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.projects.insert(
        "policy".to_string(),
        TestProjectPolicy {
            include: vec!["src/policy.test.mts".to_string()],
            integration_suites: BTreeMap::from([("openai".to_string(), Vec::new())]),
            ..Default::default()
        },
    );

    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"src/policy.test.mts".to_string()));
    assert!(rel_tests.contains(&"src/fallback.test.mts".to_string()));
}

#[test]
fn vitest_config_without_include_uses_globset_compatible_defaults() {
    let root = fixture_root("test-discovery-vitest-defaults");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));

    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert_eq!(rel_tests, vec!["src/default.test.ts"]);
    assert!(!discovered.used_fallback);
}

#[test]
fn discovered_test_globs_wraps_discovery_and_omits_empty_results() {
    let root = fixture_root("test-discovery-vitest-defaults");
    let mut config = NoMistakesConfig::default();
    config.tests.vitest.configs = Some(StringOrList::One("vitest.config.mts".to_string()));

    assert_eq!(
        discovered_test_globs(&root, &config, TestRunner::Vitest).unwrap(),
        Some(vec!["src/default.test.ts".to_string()])
    );

    let empty_root = fixture_root("symbols-output");
    assert_eq!(
        discovered_test_globs(
            &empty_root,
            &NoMistakesConfig::default(),
            TestRunner::Vitest,
        )
        .unwrap(),
        None
    );
}

#[test]
fn vitest_fallback_skips_playwright_policy_tests() {
    let root = fixture_root("test-discovery-policy-fallback");
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["e2e/**/*.spec.ts".to_string()],
            ..Default::default()
        },
    );

    let discovered = discover_tests(&root, &config, TestRunner::Vitest).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"src/fallback.test.mts".to_string()));
    assert!(!rel_tests.contains(&"e2e/home.spec.ts".to_string()));
}

#[test]
fn playwright_policy_exclude_prevents_generic_fallback() {
    let root = fixture_root("test-discovery-policy-fallback");
    let mut config = NoMistakesConfig::default();
    config.tests.playwright.projects.insert(
        "chromium".to_string(),
        TestProjectPolicy {
            include: vec!["e2e/**/*.spec.ts".to_string()],
            exclude: vec!["e2e/flaky.spec.ts".to_string()],
            ..Default::default()
        },
    );

    let discovered = discover_tests(&root, &config, TestRunner::Playwright).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"e2e/home.spec.ts".to_string()));
    assert!(!rel_tests.contains(&"e2e/flaky.spec.ts".to_string()));
}

#[test]
fn playwright_fallback_skips_helpers_in_e2e_directories() {
    let root = fixture_root("test-discovery-policy-fallback");
    let config = NoMistakesConfig::default();

    let discovered = discover_tests(&root, &config, TestRunner::Playwright).unwrap();

    let rel_tests: Vec<String> = discovered
        .tests
        .iter()
        .map(|path| crate::codebase::ts_source::relative_slash_path(&root, path))
        .collect();
    assert!(rel_tests.contains(&"tests/e2e/home.spec.ts".to_string()));
    assert!(!rel_tests.contains(&"tests/e2e/helpers.ts".to_string()));
}

#[test]
fn prepared_projects_share_runner_helpers_with_graph_facts_and_test_filters() {
    let source = fixture_root("prepared-test-projects");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let config =
        crate::config::v2::load_v2_config_from_visible(&root, None, &visible_paths).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible_paths)
            .unwrap();
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::from_files(
        crate::codebase::ts_source::discover_files_from_visible(&root, &[], &visible_paths),
    );
    let graph_plan = crate::codebase::dependencies::graph::GraphBuildPlan {
        imports: true,
        tests: true,
        ..Default::default()
    };
    let codebase_config = crate::codebase::config::config_from_loaded_v2(&root, None, &config);
    let preliminary = crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
        &root,
        graph_plan,
        &codebase_config,
        &config,
        &snapshot,
        crate::codebase::test_filter::TestFileFilter::fallback_only(),
    )
    .unwrap();
    let (fact_plan, mut fact_context) =
        crate::codebase::dependencies::graph::ts_fact_plan_and_context_for_plan_with_prepared(
            &root,
            graph_plan,
            &preliminary,
        );
    fact_context.set_visible_files(graph_files.visible().iter().cloned());

    crate::ast::begin_parse_count(&root);
    let prepared = prepare_test_projects_from_visible(
        &root,
        &config,
        &visible_paths,
        &tsconfig,
        graph_files.indexable(),
        fact_plan,
        fact_context.clone(),
    );
    let config_file = root.join("vitest.config.ts");
    let helper_file = root.join("vitest.projects.ts");
    assert!(prepared.graph_facts().contains_key(&config_file));
    assert!(prepared.graph_facts().contains_key(&helper_file));

    let discovered = discover_tests_from_prepared_projects(
        &root,
        &config,
        TestRunner::Vitest,
        &prepared,
        &visible_paths,
        &tsconfig,
    )
    .unwrap();
    assert!(discovered.tests.contains(&root.join("src/unit.test.ts")));
    assert!(!discovered
        .tests
        .contains(&root.join("src/excluded.test.ts")));

    let test_filter = crate::codebase::test_filter::TestFileFilter::from_prepared_projects(
        &root,
        &config,
        &visible_paths,
        prepared.project_filters(),
    );
    let prepared_graph =
        crate::codebase::dependencies::graph::prepare_graph_config_with_test_filter(
            &root,
            graph_plan,
            &codebase_config,
            &config,
            &snapshot,
            test_filter,
        )
        .unwrap();
    let mut facts = prepared.graph_facts().clone();
    let remaining = graph_files
        .indexable()
        .iter()
        .filter(|path| !facts.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    facts.extend(
        crate::codebase::ts_source::facts::collect_ts_facts_with_context(
            &remaining,
            fact_plan,
            &fact_context,
        ),
    );
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan_files_prepared_config_and_facts(
        &root,
        &tsconfig,
        graph_plan,
        &graph_files,
        None,
        &prepared_graph,
        Some(&facts),
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);

    let test_edges = [crate::codebase::dependencies::graph::EdgeKind::TestOf].into();
    let included = graph.deps_of(
        &[crate::codebase::dependencies::graph::NodeId::File(
            root.join("src/unit.test.ts"),
        )],
        Some(1),
        Some(&test_edges),
    );
    let excluded = graph.deps_of(
        &[crate::codebase::dependencies::graph::NodeId::File(
            root.join("src/excluded.test.ts"),
        )],
        Some(1),
        Some(&test_edges),
    );
    assert!(included
        .iter()
        .any(|entry| { entry.node.as_file() == Some(root.join("src/unit.ts").as_path()) }));
    assert!(excluded.is_empty());
    assert_eq!(counts.len(), 6, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn framework_preparation_plan_prepares_only_requested_runners() {
    let source = fixture_root("prepared-test-projects");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let visible_paths = snapshot.paths_for(&root);
    let sources = snapshot.source_store_for(&root);
    let config =
        crate::config::v2::load_v2_config_from_visible(&root, None, &visible_paths).unwrap();
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &visible_paths)
            .unwrap();
    let graph_plan = crate::codebase::ts_source::facts::TsFactPlan::default();
    let graph_context = crate::codebase::ts_source::facts::TsFactContext::default();

    crate::ast::begin_parse_count(&root);
    let reads_before = sources.physical_read_count();
    let unrequested = prepare_test_projects_from_visible_with_sources_and_plan(
        &root,
        &config,
        &visible_paths,
        &tsconfig,
        PreparedTestProjectRequest {
            graph: (&[], graph_plan, graph_context.clone()),
            sources: std::sync::Arc::clone(&sources),
            collect_graph_facts: false,
            preparation_plan: &FrameworkPreparationPlan::default(),
        },
    );
    assert!(unrequested.project_filters().is_empty());
    let error = unrequested
        .requested_runner_projects(TestRunner::Vitest)
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("vitest runner projects were not prepared"));
    assert_eq!(sources.physical_read_count(), reads_before);

    let mut requested = FrameworkPreparationPlan::default();
    requested.include_framework_names(["vitest", "unknown"]);
    let prepared = prepare_test_projects_from_visible_with_sources_and_plan(
        &root,
        &config,
        &visible_paths,
        &tsconfig,
        PreparedTestProjectRequest {
            graph: (&[], graph_plan, graph_context),
            sources,
            collect_graph_facts: false,
            preparation_plan: &requested,
        },
    );
    assert!(prepared
        .project_filters()
        .iter()
        .all(|(runner, _)| *runner == TestRunner::Vitest));
    assert!(prepared
        .requested_runner_projects(TestRunner::Vitest)
        .is_ok());
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&root.join("vitest.config.ts")), Some(&1));
    assert_eq!(counts.get(&root.join("vitest.projects.ts")), Some(&1));
}
