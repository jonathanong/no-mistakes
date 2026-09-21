#[test]
fn derived_resolve_check_reuses_dependency_closure() {
    let root = simple_root();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["a.mts"],
                    "relationships": ["import-static", "import-dynamic", "import-type", "workspace"],
                    "projection": "paths"
                },
                {
                    "type": "resolveCheckDependencies",
                    "dependencyReportIds": ["closure"]
                }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let derived = &value["reports"][1]["result"];
    assert_eq!(derived["allResolve"], true, "{derived}");
    assert_eq!(derived["results"].as_array().unwrap().len(), 3, "{derived}");
    assert!(derived["results"]
        .as_array()
        .unwrap()
        .iter()
        .any(|result| result["file"] == "c.mts"));
}

#[test]
fn derived_resolve_check_rejects_unknown_dependency_report_id() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": simple_root(),
            "reports": [{
                "type": "resolveCheckDependencies",
                "dependencyReportIds": ["missing"]
            }]
        })
        .to_string(),
    ))
    .unwrap_err();
    assert!(
        error
            .reason
            .contains("no dependencies report with id `missing`"),
        "{error}"
    );
}

#[test]
fn derived_resolve_check_rejects_duplicate_dependency_report_ids() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": simple_root(),
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["a.mts"],
                    "relationships": ["import-static"]
                },
                {
                    "type": "resolveCheckDependencies",
                    "dependencyReportIds": ["closure", "closure"]
                }
            ]
        })
        .to_string(),
    ))
    .unwrap_err();
    assert!(error.reason.contains("duplicate ID `closure`"), "{error}");
}

#[test]
fn derived_resolve_check_rejects_non_import_dependency_report() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": simple_root(),
            "reports": [
                {
                    "type": "dependencies",
                    "id": "mixed",
                    "files": ["a.mts"],
                    "relationships": ["call"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["mixed"] }
            ]
        })
        .to_string(),
    ))
    .unwrap_err();
    assert!(
        error
            .reason
            .contains("must use import or workspace relationships"),
        "{error}"
    );
}

#[test]
fn derived_resolve_check_rejects_default_dependency_relationships() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": simple_root(), "reports": [
            { "type": "dependencies", "id": "default", "files": ["a.mts"] },
            { "type": "resolveCheckDependencies", "dependencyReportIds": ["default"] }
        ] })
        .to_string(),
    ))
    .unwrap_err();
    assert!(
        error
            .reason
            .contains("must use import or workspace relationships"),
        "{error}"
    );
}

#[test]
fn derived_resolve_check_rejects_ignored_options() {
    let error = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": simple_root(), "reports": [
            { "type": "dependencies", "id": "closure", "files": ["a.mts"], "relationships": ["import-static"] },
            { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"], "files": ["a.mts"] }
        ] })
        .to_string(),
    ))
    .unwrap_err();
    assert!(
        error.reason.contains("does not accept option `files`"),
        "{error}"
    );
}

#[test]
fn derived_resolve_check_keeps_files_hidden_by_target_modules() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/dependencies/target-module-closure"),
    );
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "reports": [
            {
                "type": "dependencies",
                "id": "closure",
                "files": ["seed.mts"],
                "relationships": ["import-static"],
                "targetModules": ["@react/*"]
            },
            { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
        ] })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let projected = value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            entry["module"]
                .as_str()
                .or_else(|| entry["path"].as_str())
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(projected, vec!["@react/client"]);
    let derived = &value["reports"][1]["result"];
    assert_eq!(derived["allResolve"], false, "{derived}");
    let results = derived["results"].as_array().unwrap();
    let files = results
        .iter()
        .map(|result| result["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(files.contains(&"seed.mts"), "{derived}");
    assert!(files.contains(&"intermediate.mts"), "{derived}");
    let intermediate = results
        .iter()
        .find(|result| result["file"] == "intermediate.mts")
        .unwrap();
    assert!(
        intermediate["unresolved"]
            .as_array()
            .unwrap()
            .iter()
            .any(|specifier| specifier == "./missing.mts"),
        "{intermediate}"
    );
}

#[test]
fn derived_resolve_check_keeps_files_collapsed_by_folder_filters() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/folder-suffix/fixture"),
    );
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "reports": [
            { "type": "dependencies", "id": "closure", "files": ["main.mts"], "relationships": ["import-static"], "filters": ["backend/systems/*/"] },
            { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
        ] })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let collapsed = value["reports"][0]["result"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect::<Vec<_>>();
    let files = value["reports"][1]["result"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    for system in ["emails", "users", "search"] {
        assert!(collapsed.contains(&format!("backend/systems/{system}").as_str()));
        assert!(files.contains(&format!("backend/systems/{system}/index.mts").as_str()));
    }
}

#[test]
fn derived_resolve_check_matches_standalone_for_local_alias_and_external_imports() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries/fixture"),
    )
    .display()
    .to_string();
    let standalone =
        crate::napi_api::queries::resolve_check_json_impl(crate::napi_api::options::test_json_arg(
            json!({ "root": root, "files": ["broken.ts"] }).to_string(),
        ))
        .unwrap();
    let derived = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["broken.ts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let standalone: Value = serde_json::from_str(&standalone).unwrap();
    let derived: Value = serde_json::from_str(&derived).unwrap();
    assert_eq!(
        derived["reports"][1]["result"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result["file"] == "broken.ts")
            .unwrap(),
        &standalone["results"][0]
    );
}

#[test]
fn derived_resolve_check_keeps_computed_imports_unresolved() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/queries-kinds/fixture"),
    )
    .display()
    .to_string();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": ["computed.ts"],
                    "relationships": ["import-static", "import-dynamic", "import-type"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let result = &value["reports"][1]["result"]["results"][0];
    assert_eq!(result["allResolve"], false, "{result}");
    assert!(result["imports"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["computed"] == true && row["status"] == "unresolved"));
}

#[test]
fn derived_resolve_check_falls_back_for_an_automatic_invalid_tsconfig() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis/query-invalid-tsconfig"),
    )
    .display()
    .to_string();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({"root": root, "reports": [
            {"type":"dependencies","id":"closure","files":["entry.ts"],"relationships":["import-static"]},
            {"type":"resolveCheckDependencies","dependencyReportIds":["closure"]}
        ]}).to_string(),
    ))
    .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&output).unwrap()["reports"][1]["result"]["allResolve"],
        true
    );
}

#[test]
fn derived_resolve_check_honors_an_explicit_tsconfig() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/queries/nested-target-alias"),
    );
    let request = json!({"root": root, "tsconfig":"tsconfig.json", "reports": [
        {"type":"dependencies","id":"closure","files":["nested/src/alias-consumer.ts"],"relationships":["import-static"]},
        {"type":"resolveCheckDependencies","dependencyReportIds":["closure"]}
    ]});
    let derived: Value = serde_json::from_str(
        &analyze_project_json_impl(crate::napi_api::options::test_json_arg(request.to_string()))
            .unwrap(),
    )
    .unwrap();
    let standalone: Value = serde_json::from_str(&crate::napi_api::queries::resolve_check_json_impl(crate::napi_api::options::test_json_arg(json!({"root": root, "tsconfig":"tsconfig.json", "files":["nested/src/alias-consumer.ts"]}).to_string())).unwrap()).unwrap();
    assert_eq!(
        derived["reports"][1]["result"]["results"][0],
        standalone["results"][0]
    );
    assert_eq!(standalone["results"][0]["imports"][0]["status"], "external");
}

fn derived_resolve_fact_parity_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/napi/derived-resolve-fact-parity"),
    )
    .display()
    .to_string()
}

fn standalone_resolve_check(root: &str, file: &str) -> napi::Result<String> {
    crate::napi_api::queries::resolve_check_json_impl(crate::napi_api::options::test_json_arg(
        json!({ "root": root, "files": [file] }).to_string(),
    ))
}

fn derived_resolve_check(root: &str, file: &str) -> napi::Result<String> {
    analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {
                    "type": "dependencies",
                    "id": "closure",
                    "files": [file],
                    "relationships": ["import-static"]
                },
                { "type": "resolveCheckDependencies", "dependencyReportIds": ["closure"] }
            ]
        })
        .to_string(),
    ))
}

#[test]
fn derived_resolve_check_matches_standalone_for_recoverable_parse_diagnostics() {
    let root = derived_resolve_fact_parity_root();
    let standalone: Value =
        serde_json::from_str(&standalone_resolve_check(&root, "recoverable.ts").unwrap()).unwrap();
    let derived: Value =
        serde_json::from_str(&derived_resolve_check(&root, "recoverable.ts").unwrap()).unwrap();

    assert_eq!(
        derived["reports"][1]["result"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .find(|result| result["file"] == "recoverable.ts")
            .unwrap(),
        &standalone["results"][0]
    );
    assert_eq!(standalone["results"][0]["imports"][0]["status"], "resolved");
}

#[test]
fn derived_resolve_check_propagates_source_and_fatal_parse_failures() {
    let root = derived_resolve_fact_parity_root();

    for (file, expected) in [
        ("invalid-utf8.ts", "failed to read"),
        ("fatal.ts", "failed to parse"),
    ] {
        let standalone = standalone_resolve_check(&root, file).unwrap_err();
        let derived = derived_resolve_check(&root, file).unwrap_err();

        assert!(standalone.reason.contains(expected), "{standalone}");
        assert!(standalone.reason.contains(file), "{standalone}");
        assert!(derived.reason.contains(expected), "{derived}");
        assert!(derived.reason.contains(file), "{derived}");
    }
}

include!("derived_resolve_batching.rs");
