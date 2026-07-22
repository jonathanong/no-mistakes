use super::*;

fn runner_projects(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Result<Vec<ConfigProject>> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    runner_projects_from_visible(root, config, runner, &visible_paths, &tsconfig)
}

fn runner_projects_lossy(
    root: &Path,
    config: &NoMistakesConfig,
    runner: TestRunner,
) -> Vec<ConfigProject> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)
            .unwrap_or_else(|_| crate::codebase::ts_resolver::TsConfig {
                dir: root.to_path_buf(),
                paths: Vec::new(),
                paths_dir: root.to_path_buf(),
                base_url: None,
            });
    runner_projects_lossy_from_visible(root, config, runner, &visible_paths, &tsconfig)
}

fn config_project(config: &str, policy_name: &str, include: &str) -> ConfigProject {
    ConfigProject {
        config: Some(config.to_string()),
        policy_name: Some(policy_name.to_string()),
        runner_project_arg: Some(policy_name.to_string()),
        scope: None,
        include: vec![include.to_string()],
        exclude: Vec::new(),
        vitest_setup: Vec::new(),
    }
}

#[test]
fn explicit_policy_replaces_each_matching_config_project() {
    let root = Path::new("");
    let policy = TestProjectPolicy {
        include: vec!["src/**/*.test.ts".to_string()],
        exclude: vec!["src/skip/**".to_string()],
        ..Default::default()
    };
    let mut projects = vec![
        config_project("vitest.node.ts", "shared", "node/**/*.test.ts"),
        config_project("vitest.browser.ts", "shared", "browser/**/*.test.ts"),
        config_project("vitest.other.ts", "other", "other/**/*.test.ts"),
    ];

    apply_explicit_policy_projects(
        root,
        None,
        &BTreeMap::from([("shared".to_string(), policy)]),
        &mut projects,
    );

    let shared = projects
        .iter()
        .filter(|project| project.policy_name.as_deref() == Some("shared"))
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 2);
    assert!(shared
        .iter()
        .any(|project| project.config.as_deref() == Some("vitest.node.ts")));
    assert!(shared
        .iter()
        .any(|project| project.config.as_deref() == Some("vitest.browser.ts")));
    assert_eq!(projects.len(), 3);
}

#[test]
fn swift_projects_cover_package_fallback_and_policy_overrides() {
    let root = Path::new("");
    let mut config = NoMistakesConfig::default();
    config.tests.swift.packages = vec![
        "swift-clients/missing/".to_string(),
        "swift-clients/core".to_string(),
    ];
    config.tests.swift.projects.insert(
        "swift-clients/missing".to_string(),
        TestProjectPolicy {
            include: vec!["swift-clients/missing/CustomTests/**/*.swift".to_string()],
            exclude: vec!["swift-clients/missing/CustomTests/Skipped/**/*.swift".to_string()],
            ..Default::default()
        },
    );
    config.tests.swift.projects.insert(
        "orphan-policy".to_string(),
        TestProjectPolicy {
            include: vec!["ExternalTests/**/*.swift".to_string()],
            ..Default::default()
        },
    );
    config
        .tests
        .swift
        .projects
        .insert("empty-policy".to_string(), TestProjectPolicy::default());

    let projects = runner_projects(root, &config, TestRunner::Swift).unwrap();

    assert!(projects.iter().any(|project| {
        project.policy_name.as_deref() == Some("swift-clients/missing")
            && project.config.as_deref() == Some("swift-clients/missing")
            && project.include == vec!["swift-clients/missing/CustomTests/**/*.swift"]
            && project.exclude == vec!["swift-clients/missing/CustomTests/Skipped/**/*.swift"]
    }));
    assert!(projects.iter().any(|project| {
        project.policy_name.as_deref() == Some("orphan-policy")
            && project.config.as_deref() == Some("swift-clients/missing")
            && project.include == vec!["ExternalTests/**/*.swift"]
    }));
    assert!(!projects
        .iter()
        .any(|project| project.policy_name.as_deref() == Some("empty-policy")));
}

#[test]
fn swift_projects_lossy_uses_swift_project_loader() {
    let root = Path::new("");
    let config = NoMistakesConfig::default();

    assert!(runner_projects_lossy(root, &config, TestRunner::Swift).is_empty());
}

#[test]
fn swift_projects_skip_fact_collection_for_empty_inputs_and_keep_explicit_policies() {
    let root = Path::new("");
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    };
    let policy = TestProjectPolicy {
        include: vec!["ExternalTests/**/*.swift".to_string()],
        exclude: vec!["ExternalTests/Skipped/**/*.swift".to_string()],
        ..Default::default()
    };

    let mut no_packages = NoMistakesConfig::default();
    no_packages
        .tests
        .swift
        .projects
        .insert("policy-only".to_string(), policy.clone());
    crate::codebase::swift::test_support::begin_fact_collection_count(root);
    let projects = runner_projects_from_visible(
        root,
        &no_packages,
        TestRunner::Swift,
        &[PathBuf::from("ExternalTests/Test.swift")],
        &tsconfig,
    )
    .unwrap();
    assert_eq!(
        crate::codebase::swift::test_support::finish_fact_collection_count(root),
        0
    );
    assert!(projects.iter().any(|project| {
        project.policy_name.as_deref() == Some("policy-only")
            && project.config.is_none()
            && project.include == policy.include
            && project.exclude == policy.exclude
    }));

    let mut no_visible_paths = NoMistakesConfig::default();
    no_visible_paths.tests.swift.packages = vec!["swift-client/".to_string()];
    no_visible_paths.tests.swift.projects.insert(
        "custom".to_string(),
        TestProjectPolicy {
            include: vec!["swift-client/CustomTests/**/*.swift".to_string()],
            exclude: vec!["swift-client/CustomTests/Skipped/**/*.swift".to_string()],
            ..Default::default()
        },
    );
    crate::codebase::swift::test_support::begin_fact_collection_count(root);
    let projects =
        runner_projects_from_visible(root, &no_visible_paths, TestRunner::Swift, &[], &tsconfig)
            .unwrap();
    assert_eq!(
        crate::codebase::swift::test_support::finish_fact_collection_count(root),
        0
    );
    assert!(projects.iter().any(|project| {
        project.policy_name.as_deref() == Some("custom")
            && project.config.as_deref() == Some("swift-client")
            && project.include == vec!["swift-client/CustomTests/**/*.swift"]
            && project.exclude == vec!["swift-client/CustomTests/Skipped/**/*.swift"]
    }));
}

#[test]
fn runner_config_returns_swift_policy_map() {
    let config = NoMistakesConfig::default();
    let (configs, policies) = runner_config(&config, TestRunner::Swift);

    assert!(configs.is_none());
    assert!(policies.is_empty());
}

#[test]
#[should_panic(expected = "dotnet projects are handled before runner_config")]
fn runner_config_rejects_dotnet_after_fast_path() {
    let config = NoMistakesConfig::default();

    let _ = runner_config(&config, TestRunner::Dotnet);
}
