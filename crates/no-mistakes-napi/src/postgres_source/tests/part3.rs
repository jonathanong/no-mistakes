use super::*;

#[test]
fn native_conflict_expression_binding_retains_targets_and_diagnostics() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/insert-conflict-expressions.sql"
    ));
    let facts: serde_json::Value = serde_json::from_str(
        &parse_postgres_sql_json_impl(serde_json::json!({ "sql": sql })).unwrap(),
    )
    .unwrap();
    assert_eq!(facts["diagnostics"], serde_json::json!([]));
    assert_eq!(facts["statements"].as_array().unwrap().len(), 5);
    assert_eq!(
        facts["statements"][0]["insert"]["onConflict"]["action"]["assignments"][0]["target"]
            ["subscripts"][0]["sql"],
        "values[1]"
    );
    assert_eq!(
        facts["statements"][1]["insert"]["onConflict"]["target"]["kind"],
        "expressions"
    );
    assert_eq!(
        facts["statements"][2]["insert"]["onConflict"]["action"]["assignments"][0]["target"]
            ["subscripts"][0]["sql"],
        "1"
    );
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/insert-conflict-expressions-invalid.sql"
    ));
    let facts: serde_json::Value = serde_json::from_str(
        &parse_postgres_sql_json_impl(serde_json::json!({ "sql": sql })).unwrap(),
    )
    .unwrap();
    assert_eq!(facts["diagnostics"].as_array().unwrap().len(), 15);
    assert_eq!(facts["statements"].as_array().unwrap().len(), 15);
}

#[test]
fn native_conflict_indirection_and_operator_classes_keep_enclosing_inserts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/insert-conflict-indirection.sql"
    ));
    let result: serde_json::Value = serde_json::from_str(
        &parse_postgres_sql_json_impl(serde_json::json!({ "sql": sql })).unwrap(),
    )
    .unwrap();
    assert_eq!(result["diagnostics"], serde_json::json!([]));
    assert_eq!(result["statements"].as_array().unwrap().len(), 5);
    assert_eq!(
        result["statements"][1]["insert"]["onConflict"]["action"]["assignments"][0]["target"]
            ["indirection"][1]["kind"],
        "field"
    );
    assert_eq!(
        result["statements"][2]["insert"]["onConflict"]["target"]["operatorClasses"][0]["name"]
            ["sql"],
        "text_pattern_ops"
    );
}

#[test]
fn opaque_array_callbacks_keep_both_sql_proof_flags_false() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/control-flow-array-callbacks.ts"
    ));
    let result: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": [{"fileName":"control-flow-array-callbacks.ts", "source":source}],
            "importSpecifier":"@example/db", "resolveImports":true,
            "trustedSqlTags":[{"module":"@example/db", "name":"sql"}]
        }))
        .unwrap(),
    )
    .unwrap();
    let rows = result.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["complete"], false);
    assert_eq!(rows[0]["onlyValuePlaceholders"], false);
    assert!(rows[0]["sqlText"].is_null(), "{rows:?}");
    assert_eq!(rows[0]["incompleteReason"], "unresolved-value-or-effect");
}

#[test]
fn executed_source_conditional_executor_bindings_are_lexical() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/conditional-executors.ts"
    ));
    for resolve_imports in [false, true] {
        let result: serde_json::Value = serde_json::from_str(
            &extract_executed_postgres_sql_json_impl(serde_json::json!({
                "sources": [{"fileName": "conditional-executors.ts", "source": source}],
                "importSpecifier": "@example/db",
                "executorNames": ["read", "write"],
                "resolveImports": resolve_imports
            }))
            .unwrap(),
        )
        .unwrap();
        let facts = result.as_array().unwrap();
        let annotations: Vec<_> = facts
            .iter()
            .map(|fact| fact["annotation"].as_str().unwrap())
            .collect();
        assert_eq!(
            annotations,
            [
                "assigned",
                "direct",
                "nested",
                "outer",
                "deep",
                "outside-class"
            ],
            "resolveImports={resolve_imports}: {result}"
        );
        for fact in facts {
            assert_eq!(fact["fileName"], "conditional-executors.ts");
            assert_eq!(fact["complete"], true, "{fact}");
            assert!(fact["sqlText"]
                .as_str()
                .unwrap()
                .contains("SELECT id FROM items"));
            let line = fact["line"].as_u64().unwrap() as usize;
            assert!(source
                .lines()
                .nth(line - 1)
                .unwrap()
                .contains(fact["annotation"].as_str().unwrap()));
        }
    }
}
