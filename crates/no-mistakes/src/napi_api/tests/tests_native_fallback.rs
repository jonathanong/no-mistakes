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
