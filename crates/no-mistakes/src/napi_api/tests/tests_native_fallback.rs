#[test]
fn tests_plan_json_reports_native_source_fallback() {
    let dotnet_root = format!("{}/dotnet-clients", fixture_root("dotnet-test-plan"));
    let options = json!({
        "framework": "dotnet",
        "root": dotnet_root,
        "changedFiles": ["src/App/FeedService.cs"],
    })
    .to_string();
    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], true);
    assert!(plan["fallback_reason"]
        .as_str()
        .unwrap()
        .contains("dotnet source impact"));
}

#[test]
fn tests_plan_json_reports_configured_dotnet_source_impact() {
    let dotnet_root = fixture_root("dotnet-test-plan");
    let options = json!({
        "framework": "dotnet",
        "root": dotnet_root,
        "changedFiles": ["dotnet-clients/src/App/FeedService.cs"],
    })
    .to_string();
    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "dotnet-clients/tests/App.Tests/FeedServiceTests.cs"
    );
    assert_eq!(
        plan["selected_tests"][0]["targets"][0]["runner_args"],
        json!(["dotnet-clients/tests/App.Tests/App.Tests.csproj", "--no-restore"])
    );
}

#[test]
fn tests_plan_json_reports_configured_swift_source_impact() {
    let swift_root = fixture_root("swift-test-plan");
    let options = json!({
        "framework": "swift",
        "root": swift_root,
        "changedFiles": ["swift-clients/core/Sources/VouchaCore/APIClient.swift"],
    })
    .to_string();
    let output = tests_plan_json_impl(options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let selected = plan["selected_tests"].as_array().unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert!(plan["fallback_reason"].is_null());
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().any(|test| {
        test["test_file"] == "swift-clients/core/Tests/VouchaCoreTests/APIClientTests.swift"
    }));
    assert!(selected.iter().any(|test| {
        test["test_file"] == "swift-clients/ui/Tests/VouchaUITests/RSSFeedListViewModelTests.swift"
    }));
}
