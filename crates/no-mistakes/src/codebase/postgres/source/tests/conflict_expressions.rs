use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn conflict_expressions_and_subscript_targets_retain_source_boundaries() {
    let sql = fixture("insert-conflict-expressions.sql");
    let result = facts("insert-conflict-expressions.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 5);
    for statement in &result.statements {
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
    }
    let inserts = result.statements[..4]
        .iter()
        .map(|statement| {
            let PostgresSqlStatementKind::Insert { insert } = &statement.facts else {
                panic!("insert expected")
            };
            insert
        })
        .collect::<Vec<_>>();
    for (index, expected) in [
        (0, "values[1]"),
        (2, "1"),
        (3, "coalesce(\"Values\"[1], 2)"),
    ] {
        let PostgresSqlConflictAction::DoUpdate { assignments, .. } =
            &inserts[index].on_conflict.as_ref().unwrap().action
        else {
            panic!("update expected")
        };
        let target = assignments[0].target.as_ref().unwrap();
        assert_eq!(target.subscripts[0].sql, expected);
        let span = target.subscripts[0].span.as_ref().unwrap();
        assert_eq!(&sql[span.start.offset..span.end.offset], expected);
        assert_eq!(
            assignments[0].provenance,
            PostgresSqlInsertProvenance::Literal
        );
        assert!(assignments[0].complete);
        let base = target.base.span.as_ref().unwrap();
        assert_eq!(
            &sql[base.start.offset..base.end.offset],
            if index == 3 { "\"Values\"" } else { "values" }
        );
    }
    for index in [1, 3] {
        let conflict = inserts[index].on_conflict.as_ref().unwrap();
        let PostgresSqlConflictTarget::Expressions { expressions, .. } = &conflict.target else {
            panic!("expressions expected")
        };
        assert_eq!(
            expressions[0].functions[0]
                .name
                .parts
                .last()
                .unwrap()
                .identity,
            "lower"
        );
        let span = expressions[0].span.as_ref().unwrap();
        assert_eq!(&sql[span.start.offset..span.end.offset], expressions[0].sql);
        let function_span = expressions[0].functions[0].span.as_ref().unwrap();
        assert_eq!(
            &sql[function_span.start.offset..function_span.end.offset],
            expressions[0].sql
        );
        assert_eq!(expressions[0].columns[0].parts[0].identity, "slug");
    }
    let conflict = inserts[3].on_conflict.as_ref().unwrap();
    assert_eq!(conflict.predicate.as_ref().unwrap().sql, "\"ID\" > 0");
    let PostgresSqlConflictAction::DoUpdate {
        assignments,
        predicate,
    } = &conflict.action
    else {
        panic!("update expected")
    };
    assert_eq!(assignments.len(), 3);
    assert_eq!(assignments[0].target.as_ref().unwrap().subscripts.len(), 2);
    assert!(assignments[1].target.is_none());
    let rhs = assignments[1].expression.span.as_ref().unwrap();
    assert_eq!(
        &sql[rhs.start.offset..rhs.end.offset],
        "coalesce(EXCLUDED.slug, lower('x'))"
    );
    let assignment = assignments[1].span.as_ref().unwrap();
    assert_eq!(
        &sql[assignment.start.offset..assignment.end.offset],
        "slug = coalesce(EXCLUDED.slug, lower('x'))"
    );
    assert!(predicate.is_some());
}

#[test]
fn malformed_conflict_targets_remain_diagnostic_and_keep_neighbors() {
    let result = facts("insert-conflict-expressions-invalid.sql");
    assert_eq!(result.diagnostics.len(), 15, "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 15);
    assert!(result
        .statements
        .iter()
        .all(|statement| matches!(statement.facts, PostgresSqlStatementKind::Select { .. })));
}
