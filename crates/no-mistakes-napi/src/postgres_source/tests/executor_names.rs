use super::extract_executed_postgres_sql_json_impl;

#[test]
fn extension_keeps_default_executor_calls_and_replacement_narrows() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/source.ts"
    ));
    for (key, exclusions, count) in [
        ("executorNames", vec![], 1),
        ("extraExecutorNames", vec![], 4),
        ("extraExecutorNames", vec!["read"], 3),
        ("executorNames", vec!["run"], 0),
    ] {
        let mut request = serde_json::json!({
            "sources": [{"fileName": "source.ts", "source": source}],
            "importSpecifier": "@example/db",
        });
        request[key] = serde_json::json!(["run"]);
        request["excludeExecutorNames"] = serde_json::json!(exclusions);
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

#[test]
fn empty_extension_without_module_selects_defaults_but_absent_extension_does_not() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/source.ts"
    ));
    for (extras, count) in [(false, 0), (true, 3)] {
        let mut request = serde_json::json!({
            "sources": [{"fileName": "source.ts", "source": source}], "importSpecifier": "",
        });
        if extras {
            request["extraExecutorNames"] = serde_json::json!([]);
        }
        let report: serde_json::Value =
            serde_json::from_str(&extract_executed_postgres_sql_json_impl(request).unwrap())
                .unwrap();
        assert_eq!(report.as_array().unwrap().len(), count);
    }
}

#[test]
fn excluding_all_import_names_preserves_scoped_executor_sources() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/config/postgres-executor-names/excluded-all.ts"
    ));
    let report: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": [{"fileName": "source.ts", "source": source}],
            "importSpecifier": "@example/db", "executorNames": ["run"],
            "excludeExecutorNames": ["run"],
            "executorFactoryNames": ["openTransaction"], "executorTypeNames": ["TxExecutor"],
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(report.as_array().unwrap().len(), 2, "{report}");
}

#[test]
fn forwarded_transaction_wrappers_share_native_extraction() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-no-offset/fixture/issue-3351/transaction/index.ts"
    ));
    let report: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": [{"fileName": "index.ts", "source": source}],
            "importSpecifier": "db-lib", "executorNames": ["transaction"],
            "trustedSqlTags": [{"module": "db-lib", "name": "sql"}],
        }))
        .unwrap(),
    )
    .unwrap();
    let statements = report.as_array().unwrap();
    assert_eq!(statements.len(), 2, "{report}");
    assert_eq!(
        statements[0]["sqlText"],
        "UPDATE items SET a = 1 WHERE id = $1"
    );
    assert_eq!(
        statements[1]["sqlText"],
        "UPDATE items SET a = 2 WHERE id = $1"
    );
}
