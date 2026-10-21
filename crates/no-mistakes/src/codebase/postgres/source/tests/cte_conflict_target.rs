use super::{facts, fixture};
use crate::codebase::postgres::source::*;

fn insert_of(statement: &PostgresSqlQueryStatement) -> &PostgresSqlCteInsert {
    let PostgresSqlQueryStatementKind::Insert { insert } = &statement.facts else {
        panic!("nested INSERT expected")
    };
    insert
}

#[test]
fn conflict_target_predicates_inside_ctes_stay_complete() {
    let sql = fixture("query-cte-conflict-target.sql");
    let facts = facts("query-cte-conflict-target.sql");
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
    assert_eq!(facts.statements.len(), 5);

    let PostgresSqlStatementKind::Select { query } = &facts.statements[0].facts else {
        panic!("DO NOTHING CTE expected")
    };
    assert!(query.complete, "{:?}", query.unsupported);
    let child = &query.nested_statements[0];
    assert!(child.complete, "{:?}", child.unsupported);
    let span = child.span.as_ref().unwrap();
    assert_eq!(&sql[span.start.offset..span.end.offset], child.sql);
    assert!(child.sql.contains("WHERE id > 0"));
    assert!(!child.returning.is_empty());
    let insert = insert_of(child);
    let conflict = insert.on_conflict.as_ref().unwrap();
    assert_eq!(conflict.predicate.as_ref().unwrap().sql, "id > 0");
    assert!(matches!(
        conflict.action,
        PostgresSqlConflictAction::DoNothing
    ));
    assert_eq!(facts.statements[1].sql, "SELECT 42 AS neighbor;");

    let PostgresSqlStatementKind::Select { query } = &facts.statements[2].facts else {
        panic!("DO UPDATE CTE expected")
    };
    assert!(query.complete, "{:?}", query.unsupported);
    assert_eq!(query.nested_statements.len(), 1);
    let written = query
        .ctes
        .iter()
        .find(|cte| cte.name.identity == "written")
        .unwrap();
    let seen = query
        .ctes
        .iter()
        .find(|cte| cte.name.identity == "seen")
        .unwrap();
    assert!(written.referenced);
    assert!(seen.referenced);
    let insert = insert_of(&query.nested_statements[0]);
    let conflict = insert.on_conflict.as_ref().unwrap();
    assert_eq!(conflict.predicate.as_ref().unwrap().sql, "id > 0");
    let PostgresSqlConflictAction::DoUpdate {
        assignments,
        predicate,
    } = &conflict.action
    else {
        panic!("DO UPDATE expected")
    };
    assert_eq!(assignments.len(), 1);
    assert_eq!(predicate.as_ref().unwrap().sql, "id < 9");
    assert!(!query.nested_statements[0].returning.is_empty());

    let PostgresSqlStatementKind::Select { query } = &facts.statements[3].facts else {
        panic!("DO NOTHING without RETURNING expected")
    };
    assert!(query.complete, "{:?}", query.unsupported);
    let insert = insert_of(&query.nested_statements[0]);
    assert!(query.nested_statements[0].returning.is_empty());
    assert_eq!(
        insert
            .on_conflict
            .as_ref()
            .unwrap()
            .predicate
            .as_ref()
            .unwrap()
            .sql,
        "id > 0"
    );

    let PostgresSqlStatementKind::Select { query } = &facts.statements[4].facts else {
        panic!("JOIN ON must stay distinct from ON CONFLICT")
    };
    assert!(query.complete, "{:?}", query.unsupported);
    let insert = insert_of(&query.nested_statements[0]);
    assert_eq!(
        insert
            .on_conflict
            .as_ref()
            .unwrap()
            .predicate
            .as_ref()
            .unwrap()
            .sql,
        "id > 0"
    );
    assert!(query.nested_statements[0]
        .sql
        .contains("JOIN right_table ON"));
}

#[test]
fn malformed_cte_conflict_targets_fail_closed_and_keep_the_neighbor() {
    let sql = fixture("query-cte-conflict-target-rejected.sql");
    let facts = facts("query-cte-conflict-target-rejected.sql");
    assert_eq!(facts.diagnostics.len(), 2, "{:?}", facts.diagnostics);
    assert!(facts.diagnostics.iter().all(|diagnostic| {
        diagnostic.message.contains("sql parser error")
            && diagnostic
                .span
                .as_ref()
                .is_some_and(|span| sql[span.start.offset..span.end.offset].contains("ON CONFLICT"))
    }));
    assert_eq!(
        facts
            .statements
            .iter()
            .map(|statement| statement.sql.as_str())
            .collect::<Vec<_>>(),
        ["SELECT 42 AS kept;", "SELECT 7 AS kept;"]
    );
}
