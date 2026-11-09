use super::super::{query::bound_query, Scope};
use crate::codebase::postgres::statements::{SqlBoundItemKind, SqlPinSource};
use sqlparser::{ast::Statement, dialect::GenericDialect, parser::Parser};

#[test]
fn closed_ranges_reject_anonymous_and_computed_binds_but_accept_signed_literals() {
    let sql = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("postgres-bounded-statements")
            .join("../issue-3151/fixture/closed-range-values.sql.txt"),
    )
    .unwrap();
    let statements = Parser::parse_sql(&GenericDialect {}, &sql).unwrap();
    let scope = Scope::default().with_bodies(false);
    let mut outcomes = Vec::new();
    for statement in statements {
        let Statement::Query(query) = statement else {
            panic!("expected a saved SELECT statement");
        };
        let bound = bound_query(&query, &scope, None);
        let [item] = bound.items.as_slice() else {
            panic!("expected the statement's single FROM relation");
        };
        let SqlBoundItemKind::Table(name) = &item.kind else {
            panic!("expected a physical FROM relation");
        };
        assert!(item
            .pins
            .iter()
            .any(|pin| pin.column == "group_id" && matches!(pin.source, SqlPinSource::Value)));
        let value_pin = item
            .pins
            .iter()
            .any(|pin| pin.column == "user_id" && matches!(pin.source, SqlPinSource::Value));
        outcomes.push((name.clone(), value_pin));
    }
    assert_eq!(
        outcomes,
        [
            ("anonymous_bounds".to_owned(), false),
            ("computed_bounds".to_owned(), false),
            ("positive_literals".to_owned(), true),
        ]
    );
}
