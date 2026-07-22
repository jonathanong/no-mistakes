use super::*;
use crate::tests::{Confidence, ImpactReason};

#[test]
fn unscoped_native_full_suite_fallback_requires_explicit_opt_in() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/native-fallback-opt-in"),
    );
    let test = root.join("Tests/App.Tests/ServiceTests.cs");
    let all_tests = vec![test.clone()];
    let discovered = DiscoveredTests {
        tests: all_tests.clone(),
        targets_by_path: BTreeMap::new(),
        used_fallback: false,
    };
    let changed = vec![root.join("src/App.cs")];
    let visible = no_mistakes::codebase::ts_source::discover_visible_paths(&root);

    let disabled = native_fallback_selection(
        TestFramework::Dotnet,
        &root,
        &NoMistakesConfig::default(),
        &changed,
        &[],
        &BTreeMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        &all_tests,
        &discovered,
        &visible,
        false,
        10,
    );
    assert!(disabled.is_none());

    let (_, enabled) = native_fallback_selection(
        TestFramework::Dotnet,
        &root,
        &NoMistakesConfig::default(),
        &changed,
        &[],
        &BTreeMap::new(),
        &HashSet::new(),
        &HashSet::new(),
        &all_tests,
        &discovered,
        &visible,
        true,
        10,
    )
    .expect("explicit opt-in should permit the full-suite native fallback");
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].test_file, "Tests/App.Tests/ServiceTests.cs");
}

#[test]
fn dotnet_project_fallback_reuses_prepared_visible_paths() {
    let source = include_str!("../native_fallback.rs");
    let body = source
        .split("fn dotnet_project_fallback_tests(")
        .nth(1)
        .and_then(|body| body.split("\nfn dotnet_fallback_tests(").next())
        .expect("dotnet project fallback body");

    assert!(body.contains("visible_paths"));
    assert!(!body.contains("discover_files("));
    assert!(!body.contains("discover_visible_paths("));
}

#[test]
fn native_source_detection_handles_backslash_paths() {
    let root = Path::new("/repo");
    let mut config = NoMistakesConfig::default();
    config
        .tests
        .dotnet
        .solutions
        .push("dotnet-clients/App.sln".to_string());
    assert!(is_native_source_or_project_change(
        TestFramework::Swift,
        root,
        &config,
        r"swift-clients\core\Sources\App\Config.swift"
    ));
    assert!(!is_native_source_or_project_change(
        TestFramework::Swift,
        root,
        &config,
        r"swift-clients\core\Tests\AppTests\ConfigTests.swift"
    ));
    assert!(!is_native_source_or_project_change(
        TestFramework::Swift,
        root,
        &config,
        r"swift-clients\core\tests\AppTests\ConfigTests.swift"
    ));
    assert!(is_native_source_or_project_change(
        TestFramework::Dotnet,
        root,
        &config,
        r"dotnet-clients\App.sln"
    ));
    assert!(is_native_source_or_project_change(
        TestFramework::Dotnet,
        root,
        &config,
        r"dotnet-clients\src\App\AppConfig.cs"
    ));
    assert!(!is_native_source_or_project_change(
        TestFramework::Dotnet,
        root,
        &config,
        r"dotnet-clients\tests\App.Tests\AppConfigTests.cs"
    ));
}

#[test]
fn native_changes_include_deleted_paths() {
    let root = Path::new("/repo");
    let config = NoMistakesConfig::default();
    let selected_map = BTreeMap::new();
    let changed_files = vec![PathBuf::from("/repo/src/App/App.csproj")];
    let deleted_files = vec![PathBuf::from("/repo/src/Other/Other.csproj")];
    let triggers = untraced_native_changes(
        TestFramework::Dotnet,
        root,
        &config,
        &changed_files,
        &deleted_files,
        &selected_map,
        &HashSet::new(),
    );
    assert_eq!(
        triggers,
        vec![
            PathBuf::from("/repo/src/App/App.csproj"),
            PathBuf::from("/repo/src/Other/Other.csproj"),
        ]
    );
}

#[test]
fn native_fallback_does_not_trigger_when_every_candidate_is_already_used() {
    let root = Path::new("/repo");
    let test = root.join("Tests/RootTests/APIClientTests.swift");
    let all_tests = vec![test.clone()];
    let discovered = DiscoveredTests {
        tests: all_tests.clone(),
        targets_by_path: BTreeMap::from([(
            test.clone(),
            vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                runner: "swift".to_string(),
                config: Some(String::new()),
                project: Some("RootTests".to_string()),
                base_command: vec!["swift".to_string(), "test".to_string()],
                runner_args: Vec::new(),
            }],
        )]),
        used_fallback: false,
    };
    let mut selected_map = BTreeMap::new();
    selected_map.insert(
        test.clone(),
        SelectedTest {
            test_file: relative_path(root, &test),
            confidence: Confidence::High,
            reasons: vec![ImpactReason {
                changed_file: relative_path(root, &root.join("Package.swift")),
                path: vec!["Package.swift".to_string(), relative_path(root, &test)],
                via: vec!["direct".to_string()],
                via_details: Vec::new(),
                via_details: None,
            }],
            targets: Vec::new(),
        },
    );
    let used = HashSet::from([relative_path(root, &test)]);

    assert!(native_fallback_selection(
        TestFramework::Swift,
        root,
        &NoMistakesConfig::default(),
        &[root.join("Package.swift")],
        &[],
        &selected_map,
        &HashSet::new(),
        &used,
        &all_tests,
        &discovered,
        &[],
        false,
        10,
    )
    .is_none());
}

#[test]
fn root_swift_manifest_scopes_to_root_package_tests() {
    let root = Path::new("/repo");
    let root_test = root.join("Tests/RootTests/APIClientTests.swift");
    let client_test = root.join("clients/Tests/ClientTests/APIClientTests.swift");
    let all_tests = vec![root_test.clone(), client_test.clone()];
    let discovered = DiscoveredTests {
        tests: all_tests.clone(),
        targets_by_path: BTreeMap::from([
            (
                root_test.clone(),
                vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                    runner: "swift".to_string(),
                    config: Some(".".to_string()),
                    project: Some("RootTests".to_string()),
                    base_command: vec!["swift".to_string(), "test".to_string()],
                    runner_args: Vec::new(),
                }],
            ),
            (
                client_test,
                vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                    runner: "swift".to_string(),
                    config: Some("clients".to_string()),
                    project: Some("ClientTests".to_string()),
                    base_command: vec!["swift".to_string(), "test".to_string()],
                    runner_args: Vec::new(),
                }],
            ),
        ]),
        used_fallback: false,
    };

    assert_eq!(
        swift_manifest_fallback_tests(root, &root.join("Package.swift"), &all_tests, &discovered),
        vec![root_test]
    );
}

#[test]
fn dotnet_project_test_flag_marks_explicit_test_projects() {
    let root = Path::new("/repo");
    let mut config = NoMistakesConfig::default();
    config.tests.dotnet.projects.insert(
        "manual-tests".to_string(),
        no_mistakes::config::v2::schema::DotnetProjectConfig {
            project: "tests/Manual/Manual.csproj".to_string(),
            include: Vec::new(),
            exclude: Vec::new(),
            test: true,
        },
    );
    let project_path = PathBuf::from("/repo/tests/Manual/Manual.csproj");
    let explicit = explicit_dotnet_test_projects(root, &config);
    let facts = no_mistakes::codebase::dotnet::DotnetProjectFacts {
        project_path,
        is_test: false,
        ..Default::default()
    };

    assert!(dotnet_project_is_test(&facts, &explicit));
}

#[test]
fn target_config_matching_normalizes_dot_prefixes() {
    let root = Path::new("/repo");
    let test = root.join("clients/tests/App.Tests/AppServiceTests.cs");
    let all_tests = vec![test.clone()];
    let discovered = DiscoveredTests {
        tests: all_tests.clone(),
        targets_by_path: BTreeMap::from([(
            test.clone(),
            vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                runner: "dotnet".to_string(),
                config: Some("./clients/tests/App.Tests/App.Tests.csproj".to_string()),
                project: Some("Company.App.Tests".to_string()),
                base_command: vec!["dotnet".to_string(), "test".to_string()],
                runner_args: Vec::new(),
            }],
        )]),
        used_fallback: false,
    };

    assert_eq!(
        tests_with_target_configs(
            &all_tests,
            &discovered,
            ["clients/tests/App.Tests/App.Tests.csproj".to_string()]
        ),
        vec![test]
    );
}

#[test]
fn scoped_fallback_preserves_unscoped_discovered_tests() {
    let root = Path::new("/repo");
    let scoped = root.join("clients/tests/App.Tests/AppServiceTests.cs");
    let unscoped = root.join("fallback/UnknownTests.cs");
    let all_tests = vec![scoped.clone(), unscoped.clone()];
    let discovered = DiscoveredTests {
        tests: all_tests.clone(),
        targets_by_path: BTreeMap::from([
            (
                scoped.clone(),
                vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                    runner: "dotnet".to_string(),
                    config: Some("clients/tests/App.Tests/App.Tests.csproj".to_string()),
                    project: Some("Company.App.Tests".to_string()),
                    base_command: vec!["dotnet".to_string(), "test".to_string()],
                    runner_args: Vec::new(),
                }],
            ),
            (
                unscoped.clone(),
                vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                    runner: "dotnet".to_string(),
                    config: None,
                    project: None,
                    base_command: vec!["dotnet".to_string(), "test".to_string()],
                    runner_args: Vec::new(),
                }],
            ),
        ]),
        used_fallback: false,
    };

    assert_eq!(
        tests_with_target_configs(
            &all_tests,
            &discovered,
            ["clients/tests/App.Tests/App.Tests.csproj".to_string()]
        ),
        vec![scoped, unscoped]
    );
}

#[test]
fn empty_scoped_configs_do_not_select_unscoped_tests() {
    let root = Path::new("/repo");
    let unscoped = root.join("fallback/UnknownTests.cs");
    let all_tests = vec![unscoped.clone()];
    let discovered = DiscoveredTests {
        tests: all_tests.clone(),
        targets_by_path: BTreeMap::from([(
            unscoped,
            vec![no_mistakes::codebase::test_discovery::TestExecutionTarget {
                runner: "dotnet".to_string(),
                config: None,
                project: None,
                base_command: vec!["dotnet".to_string(), "test".to_string()],
                runner_args: Vec::new(),
            }],
        )]),
        used_fallback: false,
    };

    assert!(
        tests_with_nonempty_target_configs(&all_tests, &discovered, Vec::<String>::new())
            .is_empty()
    );
}
