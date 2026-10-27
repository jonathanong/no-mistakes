#[test]
fn nonrecursive_cte_base_reads_do_not_invent_outer_row_dependence() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nonrecursive-cte-visibility.sql"));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
}

#[path = "returning_ctes.rs"]
mod returning_ctes;
