use super::*;

pub(super) fn prepare_test_projects_from_visible(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    graph_indexable_files: &[PathBuf],
    graph_plan: crate::codebase::ts_source::facts::TsFactPlan,
    graph_context: crate::codebase::ts_source::facts::TsFactContext,
) -> PreparedTestProjects {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, visible_paths);
    prepare_test_projects_from_visible_with_sources_and_plan(
        root,
        config,
        visible_paths,
        std::sync::Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
            root,
            tsconfig.clone(),
            None,
        )),
        PreparedTestProjectRequest {
            graph: (graph_indexable_files, graph_plan, graph_context),
            sources: snapshot.source_store_for(root),
            collect_graph_facts: true,
            preparation_plan: &FrameworkPreparationPlan::all(),
        },
    )
}

#[test]
fn dotnet_strict_project_discovery_errors_on_missing_projects() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.tests.dotnet.projects.insert(
        "missing".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/tests/Missing/Missing.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );
    let error = runner_projects(&root, &config, TestRunner::Dotnet).unwrap_err();
    assert!(error
        .to_string()
        .contains("configured dotnet project `missing`"));
}

#[test]
fn dotnet_lossy_project_discovery_skips_missing_projects() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.tests.dotnet.projects.insert(
        "missing".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/tests/Missing/Missing.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );
    let projects = runner_projects_lossy(&root, &config, TestRunner::Dotnet);
    assert!(projects
        .iter()
        .any(|project| project.policy_name.as_deref() == Some("app-tests")));
    assert!(!projects
        .iter()
        .any(|project| project.policy_name.as_deref() == Some("missing")));
}

#[test]
fn dotnet_project_discovery_honors_include_override() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config
        .tests
        .dotnet
        .projects
        .get_mut("app-tests")
        .expect("fixture should define app-tests")
        .include = vec!["dotnet-clients/tests/App.Tests/ParserEdgeCases.cs".to_string()];
    let projects = runner_projects(&root, &config, TestRunner::Dotnet).unwrap();
    let app_tests = projects
        .iter()
        .find(|project| project.policy_name.as_deref() == Some("app-tests"))
        .expect("app-tests project should be discovered");
    assert_eq!(
        app_tests.include,
        vec!["dotnet-clients/tests/App.Tests/ParserEdgeCases.cs"]
    );
}

#[test]
fn dotnet_project_discovery_falls_back_when_no_xunit_files_are_known() {
    let root = fixture_root("dotnet-test-plan");
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config.tests.dotnet.solutions.clear();
    config.tests.dotnet.projects.clear();
    config.tests.dotnet.projects.insert(
        "fallback".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/src/Fallback/Fallback.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );
    let projects = runner_projects(&root, &config, TestRunner::Dotnet).unwrap();
    assert_eq!(projects.len(), 1);
    assert_eq!(
        projects[0].include,
        vec!["dotnet-clients/src/Fallback/**/*.cs"]
    );
}

#[test]
fn test_runner_framework_maps_dotnet_and_swift() {
    assert_eq!(
        TestRunner::Dotnet.framework(),
        crate::integration_tests::types::Framework::Dotnet
    );
    assert_eq!(
        TestRunner::Swift.framework(),
        crate::integration_tests::types::Framework::Swift
    );
}

#[test]
fn vitest_project_discovery_without_playwright_projects_keeps_matching_tests() {
    let root = fixture_root("symbols-output");
    let config = NoMistakesConfig::default();
    let projects = vec![ConfigProject {
        config: Some("vitest.config.mts".to_string()),
        workspace: false,
        policy_name: Some("all-specs".to_string()),
        runner_project_arg: Some("all-specs".to_string()),
        scope: None,
        include: vec!["src/utils.mts".to_string()],
        exclude: Vec::new(),
        vitest_setup: Vec::new(),
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
fn framework_preparation_plan_expands_only_required_runner_dependencies() {
    let native_only =
        FrameworkPreparationPlan::for_graph(crate::codebase::dependencies::graph::GraphBuildPlan {
            dotnet: true,
            swift: true,
            ..Default::default()
        });
    assert_eq!(native_only.runners().count(), 0);

    let tests =
        FrameworkPreparationPlan::for_graph(crate::codebase::dependencies::graph::GraphBuildPlan {
            tests: true,
            ..Default::default()
        });
    assert_eq!(tests.runners().count(), 4);

    let vitest = FrameworkPreparationPlan::for_runners([TestRunner::Vitest]);
    assert!(vitest.contains(TestRunner::Vitest));
    assert!(vitest.contains(TestRunner::Playwright));
    assert!(!vitest.contains(TestRunner::Dotnet));
    assert!(!vitest.contains(TestRunner::Swift));

    for graph_plan in [
        crate::codebase::dependencies::graph::GraphBuildPlan {
            routes: true,
            ..Default::default()
        },
        crate::codebase::dependencies::graph::GraphBuildPlan {
            http: true,
            ..Default::default()
        },
    ] {
        let plan = FrameworkPreparationPlan::for_graph(graph_plan);
        assert!(plan.contains(TestRunner::Vitest));
        assert!(plan.contains(TestRunner::Playwright));
        assert!(!plan.contains(TestRunner::Dotnet));
        assert!(!plan.contains(TestRunner::Swift));
    }
}
