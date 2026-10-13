use super::extract_executed_postgres_sql_json_impl;
use super::parse_postgres_sql_json_impl;

#[test]
fn executed_source_binding_keeps_annotation_text_and_source_location() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/reads.ts"
    ));
    let result: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": [{ "fileName": "reads.ts", "source": source }],
            "importSpecifier": "@example/db"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result.as_array().unwrap().len(), 2);
    assert_eq!(result[0]["fileName"], "reads.ts");
    assert_eq!(result[0]["line"], 4);
    assert_eq!(result[0]["annotation"], "posts/by-id");
    assert_eq!(result[0]["complete"], false);
    assert_eq!(result[0]["kind"], "dynamic");
    assert_eq!(result[1]["kind"], "inline");
    assert!(result[0]["sqlText"]
        .as_str()
        .unwrap()
        .contains("sql_placeholder_1"));
}

#[test]
fn executed_source_binding_rejects_parse_diagnostics_and_unknown_options() {
    let valid = serde_json::json!({
        "fileName": "reads.ts",
        "source": include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/postgres-facts/executed-source/reads.ts"
        ))
    });
    let broken = serde_json::json!({
        "fileName": "broken.ts",
        "source": include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/postgres-facts/executed-source/broken.ts"
        ))
    });
    let parse_error = extract_executed_postgres_sql_json_impl(serde_json::json!({
        "sources": [valid.clone(), broken],
        "importSpecifier": "@example/db"
    }))
    .unwrap_err();
    assert!(
        parse_error.to_string().contains("broken.ts"),
        "{parse_error}"
    );
    let option_error = extract_executed_postgres_sql_json_impl(serde_json::json!({
        "sources": [valid],
        "importSpecifier": "@example/db",
        "executorFactoryName": ["makeQuery"]
    }))
    .unwrap_err();
    assert!(
        option_error.to_string().contains("executorFactoryName"),
        "{option_error}"
    );
}

#[test]
fn executed_source_binding_opts_in_to_supplied_import_resolution() {
    let consumer = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/aliased-consumer.ts"
    ));
    let builder = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/imported-builder.ts"
    ));
    let sources = serde_json::json!([
        {"fileName":"aliased-consumer.ts","source":consumer},
        {"fileName":"imported-builder.ts","source":builder}
    ]);
    let unresolved: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": sources,
            "importSpecifier":"@example/db"
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(unresolved[0]["sqlText"].is_null());
    let edges = serde_json::json!([{
        "fromFileName":"aliased-consumer.ts",
        "specifier":"@example/builder",
        "toFileName":"imported-builder.ts"
    }]);
    let resolved: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": sources,
            "importSpecifier":"@example/db",
            "resolveImports":true,
            "resolvedImports": edges
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(resolved[0]["annotation"], "imported/read");
    assert_eq!(resolved[0]["complete"], true);
    let error = extract_executed_postgres_sql_json_impl(serde_json::json!({
        "sources": sources,
        "importSpecifier":"@example/db",
        "resolvedImports": edges
    }))
    .unwrap_err();
    assert!(error.to_string().contains("requires resolveImports"));
}

#[test]
fn supplied_import_projection_keeps_authored_text_and_effects_distinct() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/review-safety-consumer.ts"
    ));
    let helper = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/executed-source/review-safety-helper.ts"
    ));
    let result: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": [
                {"fileName":"review-safety-consumer.ts","source":source},
                {"fileName":"review-safety-helper.ts","source":helper}
            ],
            "importSpecifier":"@example/db",
            "resolveImports":true
        }))
        .unwrap(),
    )
    .unwrap();
    let rows = result.as_array().unwrap();
    let by_annotation = |name: &str| {
        rows.iter()
            .find(|row| row["annotation"] == name)
            .unwrap_or_else(|| panic!("missing {name}: {rows:?}"))
    };
    assert_eq!(by_annotation("authored marker")["complete"], true);
    assert_eq!(
        by_annotation("authored marker")["onlyValuePlaceholders"],
        false
    );
    assert_eq!(by_annotation("generated marker")["complete"], false);
    assert_eq!(
        by_annotation("generated marker")["onlyValuePlaceholders"],
        true
    );
    let two_binds = by_annotation("two binds");
    assert_eq!(two_binds["onlyValuePlaceholders"], true);
    let sql = two_binds["sqlText"].as_str().unwrap();
    assert!(sql.contains("sql_placeholder_1::text || sql_placeholder_2::text"));
    for (name, expected_kind) in [("bound read", None), ("bound write", Some("update"))] {
        let row = by_annotation(name);
        assert_eq!(row["onlyValuePlaceholders"], true);
        let parsed: serde_json::Value = serde_json::from_str(
            &parse_postgres_sql_json_impl(serde_json::json!({"sql":row["sqlText"]})).unwrap(),
        )
        .unwrap();
        assert!(parsed["diagnostics"].as_array().unwrap().is_empty());
        assert_eq!(parsed["statements"][0]["kind"], "select");
        let nested = parsed["statements"][0]["query"]["nestedStatements"]
            .as_array()
            .unwrap();
        assert_eq!(
            nested
                .first()
                .and_then(|statement| statement["kind"].as_str()),
            expected_kind
        );
    }
    for line in [22, 28, 40, 60, 73] {
        assert!(
            rows.iter().any(|row| {
                row["line"] == line
                    && row["complete"] == false
                    && row["onlyValuePlaceholders"] == false
                    && row["annotation"].is_null()
            }),
            "line {line}: {rows:?}"
        );
    }
}
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
