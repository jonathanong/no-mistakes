use crate::codebase::postgres::{extract_sql_statement_facts, SqlPinSource};

#[test]
fn nonrecursive_cte_definitions_keep_base_relation_ownership() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/nonrecursive-cte-visibility.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 5);
    for index in [0, 1, 2, 3] {
        let pins = &facts.bounds[index].query.items[0].pins;
        assert_eq!(pins.len(), 1, "statement {index} retains its subquery pin");
        assert!(matches!(pins[0].source, SqlPinSource::Query(_)));
        assert!(pins[0].reads.iter().all(|read| !read.tables.is_empty()));
    }
    assert_eq!(facts.bounds[4].query.items[0].pins.len(), 1);
    let pin = &facts.bounds[4].query.items[0].pins[0];
    assert!(matches!(pin.source, SqlPinSource::Query(_)));
    assert!(!pin.qualified_reads.is_empty());
}
