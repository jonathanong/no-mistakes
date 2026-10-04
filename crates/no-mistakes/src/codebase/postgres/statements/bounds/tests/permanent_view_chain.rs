use super::shape;

#[test]
fn permanent_view_chains_invalidate_temporary_dependents_only_on_matching_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-chain.sql"));
    let parsed = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!parsed.parse_failed);
    assert_eq!(parsed.bounds.len(), 10);
    assert_eq!(
        shape(sql),
        [
            "select: public.outer_view",
            "select: opaque",
            "select: opaque",
            "select: opaque",
            "select: opaque",
            "select: orders",
            "select: opaque",
            "select: orders",
            "select: orders",
            "select: orders",
        ]
    );
}

#[test]
fn physical_view_sources_follow_only_the_matching_rename() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-rename.sql"));
    let parsed = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!parsed.parse_failed);
    assert_eq!(shape(sql), ["select: opaque", "select: orders"]);
}

#[test]
fn permanent_view_chain_follows_schema_rename_before_cascade() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-schema-rename.sql"));
    let parsed = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!parsed.parse_failed);
    assert_eq!(shape(sql), ["select: opaque", "select: orders"]);
}

#[test]
fn ambiguous_source_rename_retains_original_permanent_view_edge() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-permanent-view-ambiguous-rename.sql"));
    let parsed = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!parsed.parse_failed);
    assert_eq!(shape(sql), ["select: orders"]);
}
