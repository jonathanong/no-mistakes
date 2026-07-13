use super::project::{
    evaluate_project_with_program, finalize_project_facts, parse_msbuild_json,
    parse_msbuild_output, parse_project,
};
use super::project_static::parse_project_static;
use super::*;
use std::collections::BTreeSet;

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
fn csharp_parser_ignores_comments_and_csharp_string_forms() {
    let path = fixture().join("dotnet-clients/tests/App.Tests/ParserEdgeCases.cs");
    let facts = parse_csharp_file(&path, None).expect("fixture should parse");

    assert_eq!(facts.namespace.as_deref(), Some("Company.App.Tests"));
    assert!(facts.usings.contains(&"System.Text".to_string()));
    assert!(!facts.usings.iter().any(|using| using.contains("Hidden")));
    assert!(facts.declarations.contains(&"ParserEdgeCases".to_string()));
    assert!(facts
        .references
        .contains(&"ParserLocalReference".to_string()));
    assert!(!facts.references.contains(&"CommentedReference".to_string()));
}

#[test]
fn csharp_parser_handles_block_scoped_namespaces() {
    let path = fixture().join("dotnet-clients/src/App/BlockNamespace.cs");
    let facts = parse_csharp_file(&path, None).expect("fixture should parse");

    assert_eq!(facts.namespace.as_deref(), Some("Company.App"));
    assert!(facts.usings.contains(&"Company.App".to_string()));
    assert!(facts.declarations.contains(&"BlockNamespace".to_string()));
    assert!(facts.references.contains(&"FeedService".to_string()));
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
fn project_static_parser_uses_defaults_and_test_sdk_signal() {
    let path = fixture().join("dotnet-clients/src/Fallback/Fallback.csproj");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = parse_project_static(&path, &source);

    assert!(facts.is_test);
    assert_eq!(facts.assembly_name, "Fallback");
    assert_eq!(facts.root_namespace, "Fallback");
}

#[test]
fn project_static_parser_handles_missing_file_stem_and_empty_tags() {
    let facts = parse_project_static(Path::new("/"), "<Project><PropertyGroup /></Project>");

    assert_eq!(facts.assembly_name, "Project");
    assert_eq!(facts.root_namespace, "Project");
    assert!(!facts.is_test);
    assert!(facts.compile_files.is_empty());
    assert!(facts.project_references.is_empty());
    assert!(facts.package_references.is_empty());
}

#[test]
fn msbuild_json_parser_extracts_json_from_cli_banners() {
    let project_path = fixture().join("dotnet-clients/tests/App.Tests/App.Tests.csproj");
    let json = r#"
warning banner
{
  "Properties": {
    "AssemblyName": "App.Tests",
    "RootNamespace": "Company.App.Tests",
    "IsTestProject": "true"
  },
  "Items": {
    "Compile": [{ "Identity": "FeedServiceTests.cs" }],
    "ProjectReference": [{ "Identity": "..\\..\\src\\App\\App.csproj" }],
    "PackageReference": [{ "Identity": "xunit.v3" }]
  }
}
trailing banner
"#;
    let facts = parse_msbuild_json(&project_path, json).expect("json should parse");

    assert!(facts.is_test);
    assert_eq!(facts.root_namespace, "Company.App.Tests");
    assert!(facts
        .compile_files
        .iter()
        .any(|path| path.ends_with("FeedServiceTests.cs")));
    assert!(facts.package_references.contains("xunit.v3"));
}

#[test]
fn msbuild_json_parser_handles_missing_items_and_lowercase_identity() {
    let project_path = fixture().join("dotnet-clients/src/App/App.csproj");
    assert!(parse_msbuild_json(&project_path, "not json").is_none());
    assert!(parse_msbuild_json(
        &project_path,
        r#"{ "properties": { "AssemblyName": "App" } }"#
    )
    .expect("json without items should parse")
    .compile_files
    .is_empty());

    let json = r#"{
      "properties": {
        "AssemblyName": "App",
        "RootNamespace": "Company.App"
      },
      "items": {
        "Compile": [{ "identity": "FeedService.cs" }]
      }
    }"#;
    let facts = parse_msbuild_json(&project_path, json).expect("json should parse");

    assert!(!facts.is_test);
    assert!(facts
        .compile_files
        .iter()
        .any(|path| path.ends_with("FeedService.cs")));
    assert!(facts.project_references.is_empty());
    assert!(facts.package_references.is_empty());
}

#[test]
fn msbuild_json_parser_ignores_items_without_identity() {
    let project_path = fixture().join("dotnet-clients/src/App/App.csproj");
    let json = r#"{
      "Properties": {
        "AssemblyName": "App",
        "RootNamespace": "Company.App"
      },
      "Items": {
        "Compile": [{}, { "Identity": "FeedService.cs" }],
        "ProjectReference": [{}],
        "PackageReference": [{}]
      }
    }"#;
    let facts = parse_msbuild_json(&project_path, json).expect("json should parse");

    assert!(facts
        .compile_files
        .iter()
        .any(|path| path.ends_with("FeedService.cs")));
    assert!(facts.project_references.is_empty());
    assert!(facts.package_references.is_empty());
}

#[test]
fn msbuild_evaluation_reports_start_status_and_parse_failures() {
    let project_path = fixture().join("dotnet-clients/src/App/App.csproj");
    let start_error = evaluate_project_with_program(
        &project_path,
        "app",
        "/definitely-missing-no-mistakes-dotnet",
    )
    .expect_err("missing msbuild executable must fail");
    assert!(start_error.contains("dotnet msbuild failed to start for `app`"));

    let status_error = parse_msbuild_output(
        &project_path,
        "app",
        false,
        b"",
        b"synthetic msbuild failure\n",
    )
    .expect_err("failed msbuild status must fail");
    assert_eq!(
        status_error,
        "dotnet msbuild failed for `app`: synthetic msbuild failure"
    );

    let parse_error = parse_msbuild_output(&project_path, "app", true, b"banner only", b"")
        .expect_err("unparseable successful output must fail");
    assert_eq!(
        parse_error,
        "dotnet msbuild output was not parseable for `app`"
    );
}

#[test]
fn project_finalize_fills_defaults_and_filters_compile_files() {
    let root = normalize_path(&fixture());
    let project_path = normalize_path(&root.join("dotnet-clients/src/Fallback/Fallback.csproj"));
    let project_dir = normalize_path(&root.join("dotnet-clients/src/Fallback"));
    let source_file = normalize_path(&project_dir.join("FallbackService.cs"));
    let outside_file = PathBuf::from("/outside/FallbackService.cs");
    let config = DotnetConfigProject {
        name: "fallback".to_string(),
        project: "dotnet-clients/src/Fallback/Fallback.csproj".to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        test: true,
    };
    let mut facts = DotnetProjectFacts::default();

    finalize_project_facts(
        &mut facts,
        &root,
        &[source_file.clone(), outside_file],
        &config,
        &project_path,
        &project_dir,
    );

    assert!(facts.is_test);
    assert_eq!(facts.name, "fallback");
    assert_eq!(facts.assembly_name, "Fallback");
    assert_eq!(facts.root_namespace, "Fallback");
    assert_eq!(
        facts.compile_files.into_iter().collect::<Vec<_>>(),
        vec![source_file]
    );
}

#[test]
fn project_finalize_drops_msbuild_compile_items_missing_from_visible_candidates() {
    let fixture = crate::test_support::materialize_gitignore_fixture("pass3-visibility");
    crate::test_support::git_init(fixture.path());
    crate::test_support::git_add_all(fixture.path());
    let root = normalize_path(fixture.path());
    let visible_file = root.join("dotnet/Visible.cs");
    let ignored_file = root.join("dotnet/Ignored.cs");
    let all_files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config = DotnetConfigProject {
        name: "pass3".to_string(),
        project: "dotnet/Pass3.csproj".to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        test: false,
    };
    let project_path = root.join("dotnet/Pass3.csproj");
    let project_dir = root.join("dotnet");
    let mut facts = DotnetProjectFacts {
        compile_files: BTreeSet::from([visible_file.clone(), ignored_file.clone()]),
        ..Default::default()
    };

    finalize_project_facts(
        &mut facts,
        &root,
        &all_files,
        &config,
        &project_path,
        &project_dir,
    );

    assert!(facts.compile_files.contains(&visible_file));
    assert!(!facts.compile_files.contains(&ignored_file));
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
fn parse_project_falls_back_to_static_defaults_and_file_discovery() {
    let root = fixture();
    let source_file = normalize_path(&root.join("dotnet-clients/src/Fallback/FallbackService.cs"));
    let config = DotnetConfigProject {
        name: "fallback".to_string(),
        project: "dotnet-clients/src/Fallback/Fallback.csproj".to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        test: false,
    };

    let (facts, warnings) = parse_project(&root, std::slice::from_ref(&source_file), &config);
    let facts = facts.expect("fixture project should parse");

    assert_eq!(facts.assembly_name, "Fallback");
    assert_eq!(facts.root_namespace, "Fallback");
    assert!(facts.compile_files.contains(&source_file));
    assert!(
        warnings.is_empty()
            || warnings
                .iter()
                .any(|warning| warning.contains("dotnet msbuild"))
    );
}

#[test]
fn collect_dotnet_facts_warns_about_unreadable_projects() {
    let root = fixture();
    let projects = vec![DotnetConfigProject {
        name: "missing".to_string(),
        project: "dotnet-clients/tests/Missing/Missing.csproj".to_string(),
        include: Vec::new(),
        exclude: Vec::new(),
        test: true,
    }];
    let facts = collect_dotnet_facts(&root, &[], &projects);

    assert!(facts.projects.is_empty());
    assert!(facts.warnings.iter().any(|warning| {
        warning.contains("configured dotnet project `missing`")
            && warning.contains("could not be read")
    }));
}

#[test]
fn collect_dotnet_facts_is_empty_without_projects() {
    let facts = collect_dotnet_facts(&fixture(), &[], &[]);

    assert!(facts.projects.is_empty());
    assert!(facts.files.is_empty());
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
            include: Vec::new(),
            exclude: Vec::new(),
            test: false,
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

#[test]
fn configured_projects_ignores_unreadable_solutions() {
    let root = fixture();
    let config = crate::config::v2::schema::DotnetConfig {
        solutions: vec!["dotnet-clients/Missing.sln".to_string()],
        ..Default::default()
    };

    assert!(configured_projects(&root, &config).is_empty());
}

#[test]
fn normalize_path_collapses_current_and_parent_components() {
    let path = normalize_path(Path::new("/repo/src/../tests/./App.Tests.csproj"));

    assert_eq!(path, PathBuf::from("/repo/tests/App.Tests.csproj"));

    let relative = normalize_path(Path::new("src/../tests/./App.Tests.csproj"));
    assert_eq!(relative, PathBuf::from("tests/App.Tests.csproj"));
}
