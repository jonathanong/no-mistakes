mod common;

use common::{fixture, run, stdout};

#[test]
fn test_plan_dotnet_uses_projects_and_dependency_graph() {
    let root = fixture("dotnet-test-plan");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/src/App/FeedService.cs",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], false);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec!["dotnet-clients/tests/App.Tests/FeedServiceTests.cs"]
    );
    let target = &plan["selected_tests"][0]["targets"][0];
    assert_eq!(target["runner"], "dotnet");
    assert_eq!(target["project"], "Company.App.Tests");
    assert_eq!(
        target["config"],
        "dotnet-clients/tests/App.Tests/App.Tests.csproj"
    );
    assert_eq!(
        target["base_command"],
        serde_json::json!(["dotnet", "test"])
    );
    assert_eq!(
        target["runner_args"],
        serde_json::json!([
            "dotnet-clients/tests/App.Tests/App.Tests.csproj",
            "--no-restore"
        ])
    );
    let via: Vec<&str> = plan["selected_tests"][0]["reasons"][0]["via"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(via, vec!["dotnet"]);
}

#[test]
fn test_plan_dotnet_direct_and_coverage_error() {
    let root = fixture("dotnet-test-plan");
    let direct = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/tests/App.Tests/FeedServiceTests.cs",
        "--json",
    ]);
    assert!(
        direct.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&direct.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&direct)).unwrap();
    assert_eq!(
        plan["groups"][0]["selected"],
        serde_json::json!(["dotnet-clients/tests/App.Tests/FeedServiceTests.cs"])
    );

    let coverage = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "dotnet-clients/src/App/FeedService.cs",
        "--environment",
        "coverage-only",
        "--json",
    ]);
    assert!(!coverage.status.success());
    assert!(String::from_utf8_lossy(&coverage.stderr)
        .contains("dotnet test plans do not support the coverage group"));
}

#[test]
fn test_plan_dotnet_falls_back_when_source_graph_is_unconfigured() {
    let root = fixture("dotnet-test-plan").join("dotnet-clients");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "src/App/FeedService.cs",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("dotnet source impact"));

    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "tests/App.Tests/FeedServiceTests.cs",
            "tests/App.Tests/ParserEdgeCases.cs",
        ]
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["base_command"],
        serde_json::json!(["dotnet", "test"])
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["runner_args"],
        serde_json::json!(["--no-restore"])
    );
}

#[test]
fn test_plan_dotnet_scopes_project_file_fallback_to_referencing_tests() {
    let root = fixture("dotnet-scoped-fallback");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--changed-file",
        "clients/src/App/App.csproj",
        "--json",
    ]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(plan["fallback_triggered"], true);
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(selected, vec!["clients/tests/App.Tests/AppServiceTests.cs"]);
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["config"],
        "clients/tests/App.Tests/App.Tests.csproj"
    );
}
