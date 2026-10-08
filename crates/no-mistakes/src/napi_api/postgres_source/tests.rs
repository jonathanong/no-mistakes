use super::parse_postgres_sql_json_impl;
#[test]
fn standalone_and_batch_bindings_keep_structured_source_facts() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/schema.sql"
    ));
    let source = serde_json::json!({"sql":sql,"fileName":"schema.sql"});
    let single: serde_json::Value =
        serde_json::from_str(&parse_postgres_sql_json_impl(source.clone()).unwrap()).unwrap();
    assert_eq!(single["schemaVersion"], 1);
    assert_eq!(single["statements"][0]["kind"], "createTable");
    let batch: serde_json::Value =
        serde_json::from_str(&parse_postgres_sql_json_impl(serde_json::json!([source])).unwrap())
            .unwrap();
    assert_eq!(batch[0], single);
    assert!(parse_postgres_sql_json_impl(serde_json::json!({"fileName":"missing.sql"})).is_err());
}

#[test]
fn native_binding_projects_literal_execute_children() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/literal-execute.sql"
    ));
    let single: serde_json::Value = serde_json::from_str(
        &parse_postgres_sql_json_impl(serde_json::json!({"sql":sql})).unwrap(),
    )
    .unwrap();
    let wrapper = &single["statements"][1]["block"]["statements"][0];
    assert_eq!(wrapper["kind"], "literalExecute");
    assert_eq!(wrapper["execute"]["statements"][0]["kind"], "insert");
    assert_eq!(wrapper["execute"]["statements"][1]["kind"], "insert");
    assert_eq!(wrapper["execute"]["complete"], true);
}

#[test]
fn composite_insert_binding_preserves_syntax_and_lineage_independently() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/source/insert-composite.sql"
    ));
    let result: serde_json::Value = serde_json::from_str(
        &parse_postgres_sql_json_impl(serde_json::json!({"sql": sql})).unwrap(),
    )
    .unwrap();
    assert_eq!(result["statements"].as_array().unwrap().len(), 10);
    assert_eq!(result["diagnostics"].as_array().unwrap().len(), 1);
    for index in [0, 1, 3] {
        let insert = &result["statements"][index]["insert"];
        assert_eq!(insert["complete"], true);
        assert_eq!(
            insert["onConflict"]["action"]["assignments"][0]["provenance"],
            "derived"
        );
        assert_eq!(
            insert["onConflict"]["action"]["assignments"][0]["complete"],
            true
        );
    }
    assert_eq!(
        result["statements"][2]["insert"]["onConflict"]["action"]["assignments"][0]["provenance"],
        "excludedColumn"
    );
}

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
    assert_eq!(facts["diagnostics"].as_array().unwrap().len(), 9);
    assert_eq!(facts["statements"].as_array().unwrap().len(), 9);
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
