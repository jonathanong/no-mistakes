use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn joins_ctes_signed_literals_and_procedural_completeness_are_preserved() {
    let result = facts("insert-review.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 11);
    let inserts = result.statements[..8]
        .iter()
        .map(|statement| {
            let PostgresSqlStatementKind::Insert { insert } = &statement.facts else {
                panic!("INSERT expected")
            };
            insert
        })
        .collect::<Vec<_>>();
    assert!(inserts[0].complete);
    assert!(inserts[0].on_conflict.is_none());
    for insert in &inserts[1..3] {
        assert!(insert.complete);
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
    }
    for insert in &inserts[2..4] {
        let PostgresSqlInsertSource::Select { query, span } = &insert.source else {
            panic!("SELECT expected")
        };
        assert_eq!(query.ctes.len(), 1);
        assert!(insert.complete);
        let span = span.as_ref().unwrap();
        assert!(
            fixture("insert-review.sql")[span.start.offset..span.end.offset].starts_with("SELECT")
        );
    }
    for insert in &inserts[4..7] {
        assert!(!insert.complete);
    }
    let PostgresSqlConflictAction::DoUpdate { assignments, .. } =
        &inserts[7].on_conflict.as_ref().unwrap().action
    else {
        panic!("UPDATE expected")
    };
    assert_eq!(
        assignments
            .iter()
            .map(|assignment| assignment.provenance)
            .collect::<Vec<_>>(),
        vec![
            PostgresSqlInsertProvenance::Literal,
            PostgresSqlInsertProvenance::Literal,
            PostgresSqlInsertProvenance::Literal,
            PostgresSqlInsertProvenance::Unresolved,
            PostgresSqlInsertProvenance::Unresolved,
            PostgresSqlInsertProvenance::Unresolved,
        ]
    );
    for (statement, expected) in result.statements[8..].iter().zip([false, false, true]) {
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!("DO expected")
        };
        assert_eq!(block.complete, expected);
    }
}

#[test]
fn duplicate_conflicts_do_not_produce_complete_prefix_facts() {
    let result = facts("insert-review-invalid.sql");
    assert!(!result.diagnostics.is_empty());
    assert_eq!(result.statements.len(), 1);
    assert_eq!(result.statements[0].sql, "SELECT 42;");
}
