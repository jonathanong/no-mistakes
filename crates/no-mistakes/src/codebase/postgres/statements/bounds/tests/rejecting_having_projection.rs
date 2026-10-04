use crate::codebase::postgres::{extract_sql_statement_facts, SqlBoundItemKind, SqlPinSource};

#[test]
fn rejecting_having_removes_only_projection_expansion_evidence() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/rejecting-having-projection.sql"));
    let facts = extract_sql_statement_facts(sql);
    for line in [2, 3, 7, 14, 17] {
        let fact = facts.bounds.iter().find(|fact| fact.line == line).unwrap();
        let SqlPinSource::Query(query) = &fact.query.items[0].pins[0].source else {
            panic!("expected query pin")
        };
        assert!(
            !query
                .items
                .iter()
                .any(|item| matches!(item.kind, SqlBoundItemKind::Opaque)),
            "line {line}"
        );
    }
    for line in [6, 13] {
        let fact = facts.bounds.iter().find(|fact| fact.line == line).unwrap();
        let SqlPinSource::Query(query) = &fact.query.items[0].pins[0].source else {
            panic!("expected query pin")
        };
        assert!(!query.capped);
        assert!(query
            .items
            .iter()
            .any(|item| matches!(&item.kind, SqlBoundItemKind::Table(name) if name == "orders")));
    }
    for line in [4, 5, 12, 16, 18] {
        let fact = facts.bounds.iter().find(|fact| fact.line == line).unwrap();
        let SqlPinSource::Query(query) = &fact.query.items[0].pins[0].source else {
            panic!("expected query pin")
        };
        assert!(query
            .items
            .iter()
            .any(|item| matches!(item.kind, SqlBoundItemKind::Opaque)));
    }
}
