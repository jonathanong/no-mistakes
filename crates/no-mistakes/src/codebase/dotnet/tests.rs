use super::project_static::parse_project_static;
use super::*;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/dotnet-test-plan/fixture")
}

#[test]
fn csharp_parser_extracts_usings_declarations_refs_and_xunit_tests() {
    let path = fixture().join("dotnet-clients/tests/App.Tests/FeedServiceTests.cs");
    let facts = parse_csharp_file(&path, None).expect("fixture should parse");

    assert_eq!(facts.namespace.as_deref(), Some("Company.App.Tests"));
    assert!(facts.usings.contains(&"Company.App".to_string()));
    assert!(facts.declarations.contains(&"FeedServiceTests".to_string()));
    assert!(facts.references.contains(&"FeedService".to_string()));
    assert!(facts.has_xunit_tests);
}

#[test]
fn project_static_parser_extracts_test_project_references_and_packages() {
    let path = fixture().join("dotnet-clients/tests/App.Tests/App.Tests.csproj");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = parse_project_static(&path, &source);

    assert!(facts.is_test);
    assert_eq!(facts.root_namespace, "Company.App.Tests");
    let expected = crate::codebase::ts_resolver::normalize_path(
        &fixture().join("dotnet-clients/src/App/App.csproj"),
    );
    assert!(facts.project_references.contains(&expected));
    assert!(facts.package_references.contains("xunit.v3"));
}

#[test]
fn collect_dotnet_facts_indexes_declarations_and_test_files() {
    let root = fixture();
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let projects = configured_projects(&root, &config.tests.dotnet);
    let facts = collect_dotnet_facts(&root, &all_files, &projects);

    assert!(facts.declarations.contains_key("FeedService"));
    assert!(facts
        .files
        .values()
        .any(|file| file.has_xunit_tests && file.path.ends_with("FeedServiceTests.cs")));
    assert_eq!(facts.projects.len(), 2);
}

#[test]
fn configured_projects_adds_solution_projects_without_duplicating_explicit_projects() {
    let root = fixture();
    let mut config = crate::config::v2::schema::DotnetConfig {
        solutions: vec!["dotnet-clients/App.sln".to_string()],
        ..Default::default()
    };
    config.projects.insert(
        "app".to_string(),
        crate::config::v2::schema::DotnetProjectConfig {
            project: "dotnet-clients/src/App/App.csproj".to_string(),
            ..Default::default()
        },
    );

    let projects = configured_projects(&root, &config);
    let names = projects
        .iter()
        .map(|project| project.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["app", "App.Tests"]);
    assert!(projects
        .iter()
        .any(|project| project.project == "dotnet-clients/tests/App.Tests/App.Tests.csproj"));
}
