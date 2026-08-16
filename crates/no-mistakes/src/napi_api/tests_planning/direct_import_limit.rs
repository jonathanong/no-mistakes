#[test]
fn tests_plan_json_direct_group_keeps_same_directory_importer_under_limit() {
    let output = tests_plan_json_impl(
        json!({
            "root": fixture_root("test-plan-direct-import-limit"),
            "framework": "vitest",
            "changedFiles": ["src/dev-server.mts"],
            "environment": "prePush",
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false, "{plan:#}");
    assert_eq!(
        plan["selected_tests"]
            .as_array()
            .unwrap()
            .iter()
            .map(|test| test["test_file"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["src/dev-server.test.mts"]
    );
    assert_eq!(plan["groups"][0]["type"], "direct", "{plan:#}");
    assert_eq!(
        plan["groups"][0]["selected"],
        json!(["src/dev-server.test.mts"])
    );
}

#[test]
fn tests_plan_json_direct_group_excludes_two_hop_dependents() {
    let output = tests_plan_json_impl(
        json!({
            "root": fixture_root("test-plan-direct-import-limit"),
            "framework": "vitest",
            "changedFiles": ["src/dev-server.mts"],
            "environment": "full",
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();
    let direct = plan["groups"][0]["selected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect::<Vec<_>>();
    let dependencies = plan["groups"][1]["selected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        direct,
        ["src/dev-server.test.mts", "tests/dev-server.test.mts"]
    );
    assert!(dependencies.contains(&"src/mid.test.mts"));
    assert!(dependencies.contains(&"aaa-01.test.mts"));
    assert!(!direct.contains(&"src/mid.test.mts"));
}
