use super::super::insert::project_cte_core;
use super::super::locations::Locations;
use super::{facts, fixture};
use crate::codebase::postgres::source::dialect::PostgresSourceDialect;
use crate::codebase::postgres::source::*;
use sqlparser::ast::{Expr, SelectItem, SetExpr, Statement};
use sqlparser::parser::Parser;

fn direct_insert(facts: &PostgresSqlStatementKind) -> &PostgresSqlInsert {
    let PostgresSqlStatementKind::Insert { insert } = facts else {
        panic!("INSERT expected")
    };
    insert
}

#[test]
fn cte_core_mode_skips_returning_projection() {
    let sql = fixture("insert-cte-returning-projection.sql");
    let result = facts("insert-cte-returning-projection.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 2);

    let direct = direct_insert(&result.statements[0].facts);
    assert!(direct.complete, "{:?}", direct.diagnostics);
    assert!(direct.diagnostics.is_empty());
    assert!(matches!(
        direct.returning.as_slice(),
        [PostgresSqlReturningItem::Expression {
            alias: None,
            expression,
        }] if expression.sql == "id" && expression.children_complete
    ));

    let PostgresSqlStatementKind::Select { query } = &result.statements[1].facts else {
        panic!("CTE expected")
    };
    assert_eq!(query.nested_statements.len(), 1);
    let child = &query.nested_statements[0];
    // Collector list, not returning::project. Nested completeness belongs to #1676.
    assert!(matches!(
        child.returning.as_slice(),
        [PostgresSqlReturningItem::Expression {
            alias: None,
            expression,
        }] if expression.sql == "id BETWEEN 1 AND 2"
    ));

    let ast = Parser::parse_sql(&PostgresSourceDialect, &sql).expect("fixture SQL");
    let Statement::Query(query) = &ast[1] else {
        panic!("CTE query expected");
    };
    let SetExpr::Insert(statement) = query.with.as_ref().expect("WITH").cte_tables[0]
        .query
        .body
        .as_ref()
    else {
        panic!("CTE INSERT expected");
    };
    let Statement::Insert(value) = statement else {
        panic!("INSERT expected");
    };
    let returning = value.returning.as_deref().expect("CTE RETURNING");
    assert_eq!(returning.len(), 1);
    let SelectItem::UnnamedExpr(expr) = &returning[0] else {
        panic!("expression item expected, got {}", returning[0]);
    };
    assert!(
        matches!(expr, Expr::Between { negated: false, .. }),
        "{expr}"
    );
    let core = project_cte_core(value, &Locations::new(&sql));
    // Empty because CTE-core passes None into returning::project; forwarding this BETWEEN item would fill the list.
    assert!(core.returning.is_empty(), "{:?}", core.returning);
    assert!(core.complete, "{:?}", core.diagnostics);
    assert!(core.diagnostics.is_empty());
}
