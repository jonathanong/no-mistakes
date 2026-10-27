use crate::codebase::postgres::SchemaCatalog;

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
}
