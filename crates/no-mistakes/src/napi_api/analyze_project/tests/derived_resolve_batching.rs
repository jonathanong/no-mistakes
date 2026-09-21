fn derived_resolve_batching_root() -> String {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/napi/derived-resolve-batching"),
    )
    .display()
    .to_string()
}

fn derived_batching_reports() -> Vec<Value> {
    vec![
        json!({"type":"dependencies", "id":"static-a", "files":["src/a.ts", "src/computed.ts"], "relationships":["import-static"]}),
        json!({"type":"dependencies", "id":"static-b", "files":["src/b.ts"], "relationships":["import-static"]}),
        json!({"type":"dependencies", "id":"static-c", "files":["src/c.ts"], "relationships":["import-static"]}),
        json!({"type":"dependencies", "id":"dynamic", "files":["src/a.ts"], "relationships":["import-dynamic"]}),
        json!({"type":"dependencies", "id":"depth", "files":["src/a.ts"], "relationships":["import-static"], "depth":0}),
        json!({"type":"dependencies", "id":"filter", "files":["src/a.ts"], "relationships":["import-static"], "filters":["src/"]}),
        json!({"type":"dependencies", "id":"candidate", "files":["src/a.ts"], "relationships":["import-static"], "candidateInclude":["src/**"]}),
        json!({"type":"dependencies", "id":"paths", "files":["src/a.ts"], "relationships":["import-static"], "projection":"paths"}),
        json!({"type":"dependencies", "id":"targets", "files":["src/a.ts"], "relationships":["import-static"], "targetModules":["@scope/*"]}),
    ]
}

#[test]
fn derived_resolve_check_batches_only_compatible_closures() {
    let root = derived_resolve_batching_root();
    let reports = derived_batching_reports();
    let ids = reports
        .iter()
        .map(|report| report["id"].clone())
        .collect::<Vec<_>>();
    let mut batched_reports = reports.clone();
    batched_reports.push(json!({
        "type": "resolveCheckDependencies",
        "dependencyReportIds": ids,
    }));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({"root": root, "reports": batched_reports}).to_string(),
        ))
        .unwrap()
    };
    let value: Value = serde_json::from_str(&output).unwrap();
    let derived = &value["reports"].as_array().unwrap().last().unwrap()["result"];

    // Each dependency report still projects once. The nine references contain
    // three compatible static closures and six intentionally different
    // closures, so the derived check must issue seven additional traversals.
    assert_eq!(observer.snapshot().work["traversal.requests"], 16);

    let mut expected = std::collections::BTreeMap::new();
    for id in reports.iter().map(|report| report["id"].clone()) {
        let mut unbatched_reports = reports.clone();
        unbatched_reports.push(json!({
            "type": "resolveCheckDependencies",
            "dependencyReportIds": [id],
        }));
        let unbatched: Value = serde_json::from_str(
            &analyze_project_json_impl(crate::napi_api::options::test_json_arg(
                json!({"root": root, "reports": unbatched_reports}).to_string(),
            ))
            .unwrap(),
        )
        .unwrap();
        for result in unbatched["reports"].as_array().unwrap().last().unwrap()["result"]
            ["results"]
            .as_array()
            .unwrap()
        {
            expected.insert(result["file"].as_str().unwrap().to_string(), result.clone());
        }
    }
    assert_eq!(
        derived["results"],
        Value::Array(expected.into_values().collect()),
        "batched derived result must equal the unbatched report union"
    );
    assert_eq!(derived["allResolve"], false, "{derived}");
    let results = derived["results"].as_array().unwrap();
    assert!(results.iter().any(|result| result["file"] == "src/computed.ts" && result["imports"].as_array().unwrap().iter().any(|import| import["computed"] == true && import["status"] == "unresolved")), "{derived}");
    assert!(results.iter().any(|result| result["file"] == "src/a.ts" && result["unresolved"].as_array().unwrap().iter().any(|specifier| specifier == "./missing")), "{derived}");
    assert!(results.iter().any(|result| result["file"] == "src/dynamic-target.ts"), "{derived}");
}

#[test]
fn derived_resolve_check_keeps_different_relationship_closures_separate() {
    let root = derived_resolve_batching_root();
    let output = analyze_project_json_impl(crate::napi_api::options::test_json_arg(
        json!({
            "root": root,
            "reports": [
                {"type":"dependencies", "id":"static", "files":["src/a.ts"], "relationships":["import-static"]},
                {"type":"dependencies", "id":"dynamic", "files":["src/c.ts"], "relationships":["import-dynamic"]},
                {"type":"resolveCheckDependencies", "dependencyReportIds":["static", "dynamic"]}
            ]
        })
        .to_string(),
    ))
    .unwrap();
    let value: Value = serde_json::from_str(&output).unwrap();
    let files = value["reports"].as_array().unwrap().last().unwrap()["result"]["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|result| result["file"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(files.contains(&"src/c-dynamic-target.ts"), "{files:?}");
    assert!(
        !files.contains(&"src/dynamic-target.ts"),
        "combining static and dynamic relationships would incorrectly include a.ts's dynamic import: {files:?}"
    );
}
