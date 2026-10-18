use super::*;

#[test]
fn executed_source_binding_excludes_unowned_member_queries() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/postgres-facts/embedded/executor-provenance.ts"
    ));
    let result: serde_json::Value = serde_json::from_str(
        &extract_executed_postgres_sql_json_impl(serde_json::json!({
            "sources": [{"fileName": "executor-provenance.ts", "source": source}],
            "importSpecifier": "@example/db",
            "executorNames": ["connection"],
            "executorFactoryNames": ["openTransaction"],
            "executorTypeNames": ["TxExecutor"]
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(result.as_array().unwrap().len(), 4);
    assert_eq!(
        result
            .as_array()
            .unwrap()
            .iter()
            .map(|call| call["line"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [5, 10, 14, 19]
    );
}

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
