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
