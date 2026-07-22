fn resource_fixture_root() -> tempfile::TempDir {
    let source = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/test-plan/resource-impact"),
    );
    crate::test_support::materialize_saved_fixture(&source)
}

#[test]
fn resource_edges_are_available_through_tests_and_traversal_napi_apis() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();

    let plan = tests_plan_json_impl(
        json!({
            "root": root,
            "changedFiles": ["resources/page.txt"]
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&plan).unwrap();
    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "impact-consumer.test.ts"
    );
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["via"][0],
        "resource"
    );
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["via_details"][0]["type"],
        "resource"
    );

    let impact = tests_impact_json_impl(
        json!({
            "root": root,
            "entrypoints": ["resources/page.txt"]
        })
        .to_string(),
    )
    .unwrap();
    let impact: serde_json::Value = serde_json::from_str(&impact).unwrap();
    assert_eq!(
        impact["selected_tests"][0]["test_file"],
        "impact-consumer.test.ts"
    );

    let dependencies = dependencies_json_impl(
        json!({
            "root": root,
            "files": ["impact-consumer.ts"],
            "relationships": ["resource"]
        })
        .to_string(),
    )
    .unwrap();
    let dependencies: serde_json::Value = serde_json::from_str(&dependencies).unwrap();
    assert!(dependencies["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| {
            file["path"] == "resources/page.txt" && file["via"] == json!(["resource"])
        }));

    let dependents = dependents_json_impl(
        json!({
            "root": root,
            "files": ["resources/page.txt"],
            "relationships": ["resource"]
        })
        .to_string(),
    )
    .unwrap();
    let dependents: serde_json::Value = serde_json::from_str(&dependents).unwrap();
    assert!(dependents["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file["path"] == "impact-consumer.ts"));
}

#[test]
fn tests_plan_napi_keeps_deleted_resource_impact() {
    let fixture = resource_fixture_root();
    let root = fixture.path().canonicalize().unwrap();
    std::fs::remove_file(root.join("fixtures/schema.sql")).unwrap();
    let diff = "diff --git a/fixtures/schema.sql b/fixtures/schema.sql\n\
deleted file mode 100644\n\
--- a/fixtures/schema.sql\n\
+++ /dev/null\n\
@@ -1,1 +0,0 @@\n\
--- This tracked runtime input intentionally lives under a source-skipped directory.\n";
    let plan = tests_plan_json_impl(
        json!({
            "root": root,
            "diff": diff
        })
        .to_string(),
    )
    .unwrap();
    let plan: serde_json::Value = serde_json::from_str(&plan).unwrap();

    assert_eq!(
        plan["selected_tests"][0]["test_file"],
        "skipped-resource-consumer.test.ts"
    );
    assert_eq!(
        plan["selected_tests"][0]["reasons"][0]["changed_file"],
        "fixtures/schema.sql"
    );
}
