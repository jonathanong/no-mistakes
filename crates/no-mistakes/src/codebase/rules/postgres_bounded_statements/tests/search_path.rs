use super::offenders;
use crate::codebase::postgres::{extract_sql_statement_facts, SchemaCatalog};

#[test]
fn search_path_uses_only_explicit_schema_relation_evidence() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-search-path-evidence.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    // An omitted pg_catalog precedes even an explicitly listed pg_temp.
    assert_eq!(
        facts.bounds[0].query.items[0]
            .possible_temporary
            .as_ref()
            .unwrap()
            .earlier_schemas,
        ["pg_catalog", "missing_schema"]
    );
    let names = facts
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        ["orders", "orders", "orders", "accounts", "accounts", "accounts", "accounts"]
    );
}

#[test]
fn conditional_drop_and_rename_do_not_leave_stale_temporary_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-search-path-lifecycle.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        18
    );
    let names = extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        ["accounts", "accounts", "accounts", "accounts", "orders"]
    );
}

#[test]
fn conditional_dependencies_and_subquery_pins_keep_catalog_reads_visible() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-search-path-dependencies.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        9
    );
    let names = extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["orders", "orders"]);
}

#[test]
fn local_search_path_only_applies_inside_an_explicit_transaction() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-search-path-local.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(sql)
            .unwrap()
            .len(),
        9
    );
    let names = extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["accounts", "accounts"]);
}

#[test]
fn quoted_string_path_components_keep_commas_and_escaped_quotes() {
    let root = crate::test_support::rule_fixture_root("postgres-bounded-statements");
    let sql = std::fs::read_to_string(root.join("sql/temporary-quoted-search-path.sql")).unwrap();
    let catalog = SchemaCatalog::from_json(
        &std::fs::read_to_string(root.join("schema-quoted-search-path-evidence.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        crate::codebase::postgres::parse_postgres_sql(&sql)
            .unwrap()
            .len(),
        19
    );
    let facts = extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 7);
    assert_eq!(
        facts.bounds[0].query.items[0]
            .possible_temporary
            .as_ref()
            .unwrap()
            .earlier_schemas,
        ["pg_catalog", "empty,schema"]
    );
    assert_eq!(
        facts.bounds[1].query.items[0]
            .possible_temporary
            .as_ref()
            .unwrap()
            .earlier_schemas,
        ["pg_catalog", "odd\"schema"]
    );
    assert_eq!(
        facts.bounds[2].query.items[0]
            .possible_temporary
            .as_ref()
            .unwrap()
            .earlier_schemas,
        ["pg_catalog", "MiXeD"]
    );
    // This entire quoted value names one schema; its comma is not a path separator.
    assert!(facts.bounds[4].query.items[0].possible_temporary.is_none());
    let names = facts
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["accounts", "accounts"]);
}

#[test]
fn mixed_definite_and_ambiguous_view_sources_cannot_borrow_catalog_key() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-mixed-view-dependencies.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    let names = extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["accounts"]);
}

#[test]
fn renamed_mixed_view_reveals_catalog_namesake_after_cascade() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-mixed-view-rename.sql"
    ));
    let catalog = SchemaCatalog::from_json(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/schema-search-path-evidence.json"
    )))
    .unwrap();
    let names = extract_sql_statement_facts(sql)
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.table)
        .collect::<Vec<_>>();
    assert_eq!(names, ["orders"]);
}
