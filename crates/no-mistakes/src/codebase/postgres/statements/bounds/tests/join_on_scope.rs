use crate::codebase::postgres::{extract_sql_statement_facts, SqlPinSource};

#[test]
fn on_visibility_is_retained_as_declarative_pin_read_provenance() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/join-forward-alias.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 5);
    for (fact, outward) in facts.bounds.iter().zip([true, false, true, true, true]) {
        let pin = &fact.query.items[fact.target.unwrap()].pins[0];
        let has_outer_read = matches!(pin.source, SqlPinSource::ReadQuery(_))
            || !pin.qualified_reads.is_empty()
            || !pin.reads.is_empty();
        assert_eq!(has_outer_read, outward);
    }
}
