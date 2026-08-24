use super::*;

#[test]
fn test_plan_dotnet_deleted_native_source_triggers_fallback() {
    let root = fixture("dotnet-scoped-fallback");
    let output = run(&[
        "test",
        "plan",
        "dotnet",
        "--root",
        root.to_str().unwrap(),
        "--diff",
        root.join("delete-app-service.diff").to_str().unwrap(),
        "--global-config-fallback",
        "true",
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
        .contains("clients/src/App/DeletedService.cs"));
    let selected: Vec<&str> = plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|test| test["test_file"].as_str().unwrap())
        .collect();
    assert_eq!(
        selected,
        vec![
            "clients/tests/App.Tests/AppServiceTests.cs",
            "clients/tests/Other.Tests/OtherServiceTests.cs"
        ]
    );
}

#[test]
fn test_plan_dotnet_project_reference_reaches_dependents_without_csharp_symbol_use() {
    let fixture = semantic_fixture();
    let root = fixture.path();
    git_init(root);
    // App is intentionally absent from Core.sln: discovery must still find its test project.
    replace_from_change(root, "app-central.props", "app/Directory.Packages.props");
    let app_central = semantic_plan(root, "app/Directory.Packages.props", false);
    assert_eq!(app_central["fallback_triggered"], false);
    assert_eq!(
        group(&app_central, "dependencies"),
        vec!["app/tests/AppServiceTests.cs"]
    );
    assert_eq!(
        group(&app_central, "sample"),
        vec!["core/tests/CoreServiceTests.cs"]
    );
    assert_eq!(
        app_central["selected_tests"][0]["targets"][0]["config"],
        "app/tests/App.Tests.csproj"
    );
    replace_from_change(root, "core-central.props", "Directory.Packages.props");
    let core_central = semantic_plan(root, "Directory.Packages.props", false);
    assert_eq!(core_central["fallback_triggered"], false);
    assert_eq!(
        group(&core_central, "dependencies"),
        vec![
            "core/tests/CoreServiceTests.cs",
            "app/tests/AppServiceTests.cs"
        ]
    );
    assert!(group(&core_central, "sample").is_empty());
}

#[test]
fn test_plan_dotnet_global_package_reference_reaches_project_without_package_reference() {
    let fixture =
        saved_fixture::materialize("test-plan", "dotnet-global-package-reference/fixture");
    let root = fixture.path();
    git_init(root);
    replace_from_change(root, "Directory.Packages.props", "Directory.Packages.props");

    let plan = semantic_plan(root, "Directory.Packages.props", false);
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert_eq!(
        group(&plan, "dependencies"),
        vec!["tests/GlobalServiceTests.cs"],
        "{plan:#}"
    );
    let reason = &plan["selected_tests"][0]["reasons"][0];
    assert_eq!(reason["changed_file"], "Directory.Packages.props");
    assert!(reason["via"]
        .as_array()
        .unwrap()
        .iter()
        .all(|edge| edge == "dotnet project dependency"));
}

#[test]
fn test_plan_dotnet_semantic_project_and_lock_changes_select_only_owner_tests() {
    for (change, target) in [
        ("app-lock.json", "app/packages.lock.json"),
        ("app-project.csproj", "app/App.csproj"),
    ] {
        let fixture = semantic_fixture();
        let root = fixture.path();
        git_init(root);
        replace_from_change(root, change, target);
        let plan = semantic_plan(root, target, false);
        assert_eq!(plan["fallback_triggered"], false, "{target}: {plan:#}");
        assert_eq!(
            group(&plan, "dependencies"),
            vec!["app/tests/AppServiceTests.cs"],
            "{target}: {plan:#}"
        );
        assert_eq!(
            group(&plan, "sample"),
            vec!["core/tests/CoreServiceTests.cs"]
        );
    }
}

#[test]
fn test_plan_dotnet_formatting_only_dependency_file_selects_no_tests() {
    let fixture = semantic_fixture();
    let root = fixture.path();
    git_init(root);
    replace_from_change(root, "app-formatting.props", "app/Directory.Packages.props");

    let plan = semantic_plan(root, "app/Directory.Packages.props", false);
    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert!(group(&plan, "dependencies").is_empty(), "{plan:#}");
    assert!(plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|test| test["reasons"].as_array().unwrap())
        .all(|reason| reason["changed_file"] == "*sample*"));
}

#[test]
fn test_plan_dotnet_semantic_dependency_diagnostics_follow_global_fallback_policy() {
    for (change, expected_warning) in [
        (
            "changes/unsupported.props",
            "dotnet-dependency-unsupported-dynamic",
        ),
        (
            "app/Directory.Packages.props",
            "dotnet-dependency-no-baseline",
        ),
    ] {
        let fixture = semantic_fixture();
        let root = fixture.path();
        git_init(root);
        if change == "changes/unsupported.props" {
            replace_from_change(root, "unsupported.props", "app/Directory.Packages.props");
        }
        let plan_for = |global_fallback| {
            if change == "changes/unsupported.props" {
                semantic_plan(root, "app/Directory.Packages.props", global_fallback)
            } else {
                let output = run(&[
                    "test",
                    "plan",
                    "dotnet",
                    "--root",
                    root.to_str().unwrap(),
                    "--changed-file",
                    change,
                    "--global-config-fallback",
                    if global_fallback { "true" } else { "false" },
                    "--json",
                ]);
                serde_json::from_str(&stdout(&output)).unwrap()
            }
        };
        let disabled = plan_for(false);
        assert_eq!(
            disabled["fallback_triggered"], false,
            "{change}: {disabled:#}"
        );
        assert!(group(&disabled, "dependencies").is_empty());
        assert_eq!(group(&disabled, "sample").len(), 1);
        assert!(disabled["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning["type"] == expected_warning));
        let enabled = plan_for(true);
        assert_eq!(enabled["fallback_triggered"], true, "{change}: {enabled:#}");
        assert_eq!(enabled["selected_tests"].as_array().unwrap().len(), 2);
    }
}

#[test]
fn test_plan_dotnet_untraceable_semantic_dependency_follows_global_fallback_policy() {
    let fixture = semantic_fixture();
    let root = fixture.path();
    git_init(root);
    replace_from_change(root, "orphan-project.csproj", "orphan/Orphan.csproj");

    let disabled = semantic_plan(root, "orphan/Orphan.csproj", false);
    assert_eq!(disabled["fallback_triggered"], false, "{disabled:#}");
    assert!(group(&disabled, "dependencies").is_empty(), "{disabled:#}");
    assert_eq!(group(&disabled, "sample").len(), 1, "{disabled:#}");
    assert!(disabled["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "dotnet-dependency-untraceable"));

    let enabled = semantic_plan(root, "orphan/Orphan.csproj", true);
    assert_eq!(enabled["fallback_triggered"], true, "{enabled:#}");
    assert_eq!(enabled["selected_tests"].as_array().unwrap().len(), 2);
    assert!(enabled["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|warning| warning["type"] == "dotnet-dependency-untraceable"));
}

#[test]
fn plain_plan_dotnet_untraceable_semantic_dependency_obeys_global_fallback_policy() {
    for (fallback, expected_fallback, expected_tests) in [(false, false, 0), (true, true, 2)] {
        let fixture = semantic_fixture();
        let root = fixture.path();
        git_init(root);
        replace_from_change(root, "orphan-project.csproj", "orphan/Orphan.csproj");
        let output = run(&[
            "test",
            "plan",
            "--root",
            root.to_str().unwrap(),
            "--base",
            "HEAD",
            "--changed-file",
            "orphan/Orphan.csproj",
            "--global-config-fallback",
            if fallback { "true" } else { "false" },
            "--json",
        ]);
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let plan: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
        assert_eq!(plan["fallback_triggered"], expected_fallback, "{plan:#}");
        assert_eq!(
            plan["selected_tests"].as_array().unwrap().len(),
            expected_tests
        );
        assert!(plan["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning["type"] == "dotnet-dependency-untraceable"));
    }
}

#[test]
fn test_plan_dotnet_mixed_and_native_configuration_files_remain_broad() {
    let fixture = semantic_fixture();
    let root = fixture.path();
    git_init(root);
    replace_from_change(root, "mixed.props", "app/Directory.Packages.props");
    let mixed = semantic_plan(root, "app/Directory.Packages.props", false);
    assert_eq!(mixed["fallback_triggered"], false);
    assert_eq!(
        group(&mixed, "dependencies"),
        vec!["app/tests/AppServiceTests.cs"]
    );
    assert_eq!(
        group(&mixed, "sample"),
        vec!["core/tests/CoreServiceTests.cs"]
    );
    for changed in [
        "Directory.Build.props",
        "Directory.Build.targets",
        "NuGet.config",
        "global.json",
    ] {
        let broad = semantic_plan(root, changed, true);
        assert_eq!(broad["fallback_triggered"], true, "{changed}: {broad:#}");
        assert_eq!(broad["selected_tests"].as_array().unwrap().len(), 2);
    }
}
