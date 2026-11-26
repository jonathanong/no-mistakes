use crate::statements::{extract_sql_statement_facts_with_json_writes, SqlLocationDemand};

#[test]
fn json_row_shapes_cover_default_values_table_and_empty_from_selects() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/statements/json-row-shapes.sql");
    let sql = std::fs::read_to_string(path).unwrap();
    let facts = extract_sql_statement_facts_with_json_writes(
        &sql,
        false,
        None,
        SqlLocationDemand::None,
        true,
        false,
        false,
    );
    assert!(!facts.parse_failed);
    let keys: Vec<_> = facts
        .json_writes
        .iter()
        .map(|fact| (fact.table.as_str(), fact.key.as_str()))
        .collect();
    assert_eq!(keys, [("events", "a"), ("events", "k"), ("events", "b")]);
}
