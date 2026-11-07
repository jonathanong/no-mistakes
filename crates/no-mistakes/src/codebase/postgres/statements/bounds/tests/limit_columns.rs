use super::super::{query, Scope};
use crate::codebase::postgres::{parse_postgres_sql, SqlBoundItemKind};
use sqlparser::ast::Statement;

#[test]
fn lateral_limits_follow_caller_array_columns_and_relation_scope() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/issue-3100-limits.sql"
    ));
    let statements = parse_postgres_sql(sql).unwrap();
    let expected = [
        true, false, true, true, true, true, false, false, false, false, false, false, false, true,
        false, false, false, false, false, false, true, false, false, false, false, false, false,
        true, false, false,
    ];
    assert_eq!(statements.len(), expected.len());
    for (index, (statement, expected)) in statements.iter().zip(expected).enumerate() {
        let Statement::Query(ast) = statement else {
            panic!("saved fixture must contain queries");
        };
        let bound = query::bound_query(ast, &Scope::default(), None);
        let tested = bound
            .items
            .iter()
            .rev()
            .find_map(|item| match &item.kind {
                SqlBoundItemKind::Query(inner) => Some(inner),
                _ => None,
            })
            .unwrap_or(&bound);
        assert_eq!(tested.capped, expected, "fixture SQL line {}", index + 1);
    }
}
