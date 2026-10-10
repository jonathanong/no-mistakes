use super::facts;
use crate::codebase::postgres::source::*;

const INCOMPLETE_RETURNING: &str = "unsupported or incompletely represented syntax";

#[test]
fn incompletely_projected_cte_returning_expression_marks_query_incomplete() {
    let result = facts("insert-cte-returning-incomplete.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 2);
    // CTE-core skips `returning::project`, so incompleteness stays on the query
    // `unsupported` entry instead of a second INSERT diagnostic.
    assert_complete_addition(&result.statements[0].facts);
    assert_incomplete_between(&result.statements[1].facts);
}

fn assert_complete_addition(facts: &PostgresSqlStatementKind) {
    let query = select(facts);
    assert!(query.complete, "{:?}", query.unsupported);
    assert!(query.unsupported.is_empty());
    let child = &query.nested_statements[0];
    assert!(child.complete, "{:?}", child.unsupported);
    assert!(child.unsupported.is_empty());
    assert!(matches!(
        &child.returning[0],
        PostgresSqlReturningItem::Expression {
            alias: None,
            expression,
        } if expression.children_complete && expression.sql == "id + 1"
    ));
    assert!(
        !mentions_incomplete_syntax(child),
        "{:?}",
        child.unsupported
    );
    assert!(nested_insert(child).diagnostics.is_empty());
}

fn assert_incomplete_between(facts: &PostgresSqlStatementKind) {
    let query = select(facts);
    assert!(!query.complete);
    assert!(query
        .unsupported
        .iter()
        .any(|item| item.reason == INCOMPLETE_RETURNING));
    let child = &query.nested_statements[0];
    assert!(!child.complete);
    assert!(matches!(
        &child.returning[0],
        PostgresSqlReturningItem::Expression {
            alias: None,
            expression,
        } if !expression.children_complete && expression.sql.contains("BETWEEN")
    ));
    assert!(child.unsupported.iter().any(|item| {
        item.reason == INCOMPLETE_RETURNING && item.clause == PostgresSqlQueryClause::Projection
    }));
    assert!(nested_insert(child).diagnostics.is_empty());
}

fn mentions_incomplete_syntax(child: &PostgresSqlQueryStatement) -> bool {
    child
        .unsupported
        .iter()
        .any(|item| item.reason.contains("incompletely represented"))
}

fn select(facts: &PostgresSqlStatementKind) -> &PostgresSqlQuery {
    let PostgresSqlStatementKind::Select { query } = facts else {
        panic!("CTE query expected")
    };
    query
}

fn nested_insert(child: &PostgresSqlQueryStatement) -> &PostgresSqlCteInsert {
    let PostgresSqlQueryStatementKind::Insert { insert } = &child.facts else {
        panic!("nested INSERT expected")
    };
    insert
}
