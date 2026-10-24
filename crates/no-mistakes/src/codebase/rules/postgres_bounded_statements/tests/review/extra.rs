use super::super::names;

#[test]
fn projected_composites_ordinality_and_join_aliases_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-output-regressions.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let catalog = super::super::catalog();
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| super::super::offenders(fact, &catalog))
        .map(|finding| (finding.table, finding.line))
        .collect();
    assert_eq!(
        found,
        [12, 13, 14, 15, 23, 24].map(|line| ("accounts".to_string(), line))
    );
}

#[test]
fn projected_source_aliases_set_arms_and_unknown_functions_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-column-variants.sql"));
    assert!(names(sql).is_empty());
}

#[test]
fn qualified_scalar_functions_and_unknown_record_layouts_preserve_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-column-coverage.sql"));
    assert_eq!(names(sql), ["accounts"]);
}

#[test]
fn unnest_without_a_column_alias_exposes_its_function_or_alias_name() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/projected-unnest.sql"
    ));
    assert_eq!(names(sql), ["accounts", "accounts", "accounts", "accounts"]);
}

#[test]
fn distinct_on_set_returning_aggregate_keys_keep_their_input_unbounded() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/distinct-on-order-srf.sql"
    ));
    assert_eq!(names(sql), ["orders"]);
}
