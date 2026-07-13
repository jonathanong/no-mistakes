// Included into `napi_api::tests`; shares its fixture helpers and imports.

#[test]
fn tests_plan_why_comment_and_graph_exports_return_reports() {
    let root = fixture_root("test-plan-config");
    let plan_options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": ["source.ts"],
        "limitFiles": 1
    })
    .to_string();
    let output = tests_plan_json_impl(plan_options).unwrap();
    let plan: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(plan["fallback_triggered"], false);
    assert_eq!(plan["selected_tests"].as_array().unwrap().len(), 1);
    assert_eq!(plan["selected_tests"][0]["targets"][0]["runner"], "vitest");

    let fallback_limit_options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": ["web/app/page.tsx"],
        "limitFiles": 1
    })
    .to_string();
    let fallback_limit_output = tests_plan_json_impl(fallback_limit_options).unwrap();
    let fallback_limit: serde_json::Value = serde_json::from_str(&fallback_limit_output).unwrap();

    assert_eq!(fallback_limit["fallback_triggered"], true);
    assert_eq!(
        fallback_limit["selected_tests"].as_array().unwrap().len(),
        1
    );
    assert_eq!(fallback_limit["groups"][0]["limit"], 1);

    let no_global_fallback_options = json!({
        "framework": "vitest",
        "root": root,
        "changedFiles": [".no-mistakes.yml"],
        "globalConfigFallback": false
    })
    .to_string();
    let no_global_fallback_output = tests_plan_json_impl(no_global_fallback_options).unwrap();
    let no_global_fallback: serde_json::Value =
        serde_json::from_str(&no_global_fallback_output).unwrap();

    assert_eq!(no_global_fallback["fallback_triggered"], false);
    assert_eq!(
        no_global_fallback["selected_tests"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    let legacy_plan_options = json!({
        "root": root,
        "changedFiles": ["source.ts"],
    })
    .to_string();
    let legacy_output = tests_plan_json_impl(legacy_plan_options).unwrap();
    let legacy_plan: serde_json::Value = serde_json::from_str(&legacy_output).unwrap();

    assert_eq!(legacy_plan["fallback_triggered"], false);
    assert!(legacy_plan["selected_tests"]
        .as_array()
        .unwrap()
        .iter()
        .any(|test| test["test_file"] == "source.test.mts"));

    let comment = tests_comment_markdown_impl(json!({ "planJson": plan }).to_string()).unwrap();
    assert!(comment.contains("Selected Tests"));

    let plan_path = PathBuf::from(&root).join("plan.json");
    let path_comment =
        tests_comment_markdown_impl(json!({ "plan": plan_path.display().to_string() }).to_string())
            .unwrap();
    assert!(path_comment.contains("source.test.mts"));

    let graph = tests_graph_json_impl(json!({ "planJson": output }).to_string()).unwrap();
    let graph: serde_json::Value = serde_json::from_str(&graph).unwrap();
    assert!(!graph["nodes"].as_array().unwrap().is_empty());

    let mermaid = tests_graph_mermaid_impl(
        json!({ "planJson": serde_json::from_str::<serde_json::Value>(&output).unwrap() })
            .to_string(),
    )
    .unwrap();
    assert!(mermaid.starts_with("graph TD"));

    let why_options = json!({
        "root": fixture_root("test-plan-config"),
        "test": "source.test.mts",
        "changed": "source.ts"
    })
    .to_string();
    let why = tests_why_json_impl(why_options).unwrap();
    let why: serde_json::Value = serde_json::from_str(&why).unwrap();
    assert!(!why["source.ts"].as_array().unwrap().is_empty());
}
