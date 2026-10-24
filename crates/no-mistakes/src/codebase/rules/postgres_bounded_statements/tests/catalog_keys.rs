use super::*;

#[test]
fn quoted_key_columns_match_quoted_sql_columns() {
    let catalog = SchemaCatalog::from_json(
        r#"{
          "formatVersion": 2, "coverage": "complete",
          "tables": { "mixed": {
            "columns": { "Order Id": { "dataType": "uuid" } },
            "indexes": { "mixed_pkey": {
              "accessMethod": "btree",
              "unique": true, "primary": true, "valid": true, "ready": true,
              "keys": [ { "column": "Order Id", "expression": "\"Order Id\"" } ]
            } }
          } }
        }"#,
    )
    .unwrap();
    assert_eq!(catalog.unique_keys("mixed"), [vec!["Order Id".to_string()]]);
    let judge = |sql: &str| {
        extract_sql_statement_facts(sql)
            .bounds
            .iter()
            .flat_map(|fact| offenders(fact, &catalog))
            .count()
    };
    assert_eq!(judge("SELECT 1 FROM mixed WHERE \"Order Id\" = $1"), 0);
    assert_eq!(judge("SELECT 1 FROM mixed WHERE order_id = $1"), 1);
}
