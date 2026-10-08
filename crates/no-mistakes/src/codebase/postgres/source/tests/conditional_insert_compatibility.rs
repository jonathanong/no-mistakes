use super::{facts, fixture};
use crate::codebase::postgres::source::*;

fn insert(facts: &PostgresSqlStatementKind) -> &PostgresSqlInsert {
    let PostgresSqlStatementKind::Insert { insert } = facts else {
        panic!("INSERT expected")
    };
    insert
}

#[test]
fn conditional_wrapped_with_inserts_borrow_normalized_ast_and_preserve_source_facts() {
    let sql = fixture("conditional-with-insert.sql");
    let result = facts("conditional-with-insert.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 4);
    let baseline = insert(&result.statements[0].facts);
    assert!(baseline.complete);
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[2].facts else {
        panic!()
    };
    // VALUES lacks a place for outer WITH facts, so the body stays incomplete.
    assert!(!block.complete);
    let PostgresSqlStatementKind::Conditional { branches } = &block.statements[0].facts else {
        panic!()
    };
    let children = &branches[0].statements;
    assert_eq!(children.len(), 7);
    for index in [0, 1, 2] {
        let child = if index == 0 {
            &children[index]
        } else {
            let PostgresSqlStatementKind::Wrapper { wrapper } = &children[index].facts else {
                panic!()
            };
            assert!(wrapper.complete, "{:?}", wrapper.diagnostics);
            &wrapper.statements[0]
        };
        let projected = insert(&child.facts);
        assert!(projected.complete);
        assert_eq!(projected.table, baseline.table);
        assert_eq!(
            projected.on_conflict.as_ref().unwrap().target,
            baseline.on_conflict.as_ref().unwrap().target
        );
        assert_eq!(
            projected.on_conflict.as_ref().unwrap().action,
            baseline.on_conflict.as_ref().unwrap().action
        );
        let PostgresSqlInsertSource::Select { query, span } = &projected.source else {
            panic!()
        };
        assert_eq!(query.ctes.len(), 1);
        assert_eq!(query.ctes[0].name.identity, "src");
        assert!(query.ctes[0].referenced && query.ctes[0].used);
        let span = span.as_ref().unwrap();
        assert_eq!(
            &sql[span.start.offset..span.end.offset],
            "SELECT id FROM src"
        );
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
    }
    for index in [3, 4] {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &children[index].facts else {
            panic!()
        };
        assert!(!wrapper.complete);
        let projected = insert(&wrapper.statements[0].facts);
        assert!(!projected.complete);
        assert!(matches!(
            projected.source,
            PostgresSqlInsertSource::Values { .. }
        ));
        assert!(!projected.diagnostics.is_empty());
    }
    let PostgresSqlStatementKind::Conditional { branches } = &children[5].facts else {
        panic!()
    };
    let PostgresSqlStatementKind::Wrapper { wrapper } = &branches[0].statements[0].facts else {
        panic!()
    };
    assert!(wrapper.complete);
    let ordinary = insert(&children[6].facts);
    let ordinary_baseline = insert(&result.statements[1].facts);
    assert!(ordinary.complete);
    assert_eq!(ordinary.table, ordinary_baseline.table);
    assert_eq!(
        ordinary.on_conflict.as_ref().unwrap().target,
        ordinary_baseline.on_conflict.as_ref().unwrap().target
    );
    assert_eq!(
        ordinary.on_conflict.as_ref().unwrap().action,
        ordinary_baseline.on_conflict.as_ref().unwrap().action
    );
    assert!(matches!(
        ordinary.source,
        PostgresSqlInsertSource::Values { .. }
    ));
    assert_eq!(result.statements[3].sql, "SELECT 88;");
}

#[test]
fn conditional_partial_conflict_targets_fail_closed_and_preserve_neighbors() {
    let result = facts("conditional-partial-conflict-controls.sql");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.statements.len(), 6);
    for index in [0, 2, 4] {
        let PostgresSqlStatementKind::DoBlock { block } = &result.statements[index].facts else {
            panic!()
        };
        assert!(!block.complete);
        assert!(block.statements.is_empty());
        assert!(!block.diagnostics.is_empty());
        assert!(matches!(
            result.statements[index + 1].facts,
            PostgresSqlStatementKind::Select { .. }
        ));
    }
}
