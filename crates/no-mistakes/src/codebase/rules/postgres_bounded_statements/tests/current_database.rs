use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn explicit_database_temp_source_survives_ambiguous_namesake_in_same_view() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-mixed-current-database-view.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 1);
}

#[test]
fn temporary_database_identity_requires_explicit_catalog_match() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 16);
}

#[test]
fn qualified_temporary_views_and_folded_database_names_use_same_prepared_facts() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-views.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 9);
}

#[test]
fn database_and_search_path_evidence_must_both_match() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-search-path.sql")));
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 4);
}

#[test]
fn conditional_partition_lifetimes_reuse_facts_across_catalog_databases() {
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-partitions.sql")));
    // GenericDialect rejects ATTACH/DETACH; the narrow statement recovery is intentional.
    assert!(facts.parse_failed);
    assert_eq!(facts.bounds.len(), 6);
}

#[test]
fn wrong_kind_materialized_drops_preserve_catalog_condition() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-current-database-materialized.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
}
