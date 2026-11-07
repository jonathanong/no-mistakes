use super::extract_executed_postgres_sql_json_impl;

#[test]
fn extension_keeps_default_executor_calls_and_replacement_narrows() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/source.ts"
    ));
    for (key, count) in [("executorNames", 1), ("extraExecutorNames", 4)] {
        let mut request = serde_json::json!({
            "sources": [{"fileName": "source.ts", "source": source}],
            "importSpecifier": "@example/db",
        });
        request[key] = serde_json::json!(["run"]);
        let report: serde_json::Value =
            serde_json::from_str(&extract_executed_postgres_sql_json_impl(request).unwrap())
                .unwrap();
        assert_eq!(report.as_array().unwrap().len(), count, "{key}");
    }
}

#[test]
fn native_request_rejects_both_options_even_when_empty() {
    let error = extract_executed_postgres_sql_json_impl(serde_json::json!({
        "sources": [], "importSpecifier": "@example/db",
        "executorNames": [], "extraExecutorNames": [],
    }))
    .unwrap_err();
    assert!(error.to_string().contains("cannot both be set"));
}
