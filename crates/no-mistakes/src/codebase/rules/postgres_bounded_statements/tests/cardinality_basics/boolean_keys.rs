use super::super::fixture_root;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn bare_boolean_key_predicates_bound_only_complete_conjunctive_keys() {
    let root = fixture_root();
    let schema_path = root.join("schema-boolean-keys.json");
    let sources = crate::codebase::rules::source_store_for_files(&[schema_path]);
    let catalog = SchemaCatalog::load(&root, "schema-boolean-keys.json", &sources).unwrap();
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/boolean-keys.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 16);

    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    // OR and NOT do not yield implicit true pins; a non-key boolean is insufficient.
    assert_eq!(
        found,
        [
            ("boolean_keys", 9),
            ("boolean_keys", 10),
            ("composite_boolean_keys", 15),
            ("composite_boolean_keys", 16),
            ("composite_boolean_keys", 17),
            ("composite_boolean_keys", 20),
            ("composite_boolean_keys", 21),
            ("boolean_keys", 22),
            ("boolean_keys", 23),
        ]
        .map(|(table, line)| (table.to_owned(), line))
    );
}
