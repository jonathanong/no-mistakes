use super::super::{
    PostgresSqlAlterOperation, PostgresSqlProceduralOccurrence,
    PostgresSqlProceduralOccurrenceKind as Kind, PostgresSqlStatement, PostgresSqlStatementKind,
};
use super::{facts, fixture};

fn count_utilities(occurrences: &[PostgresSqlProceduralOccurrence]) -> usize {
    occurrences
        .iter()
        .map(|occurrence| {
            usize::from(occurrence.kind == Kind::Utility) + count_utilities(&occurrence.occurrences)
        })
        .sum()
}

fn contains_kind(occurrences: &[PostgresSqlProceduralOccurrence], kind: Kind) -> bool {
    occurrences
        .iter()
        .any(|occurrence| occurrence.kind == kind || contains_kind(&occurrence.occurrences, kind))
}

fn first_alter(statement: &PostgresSqlStatement) -> Option<&PostgresSqlStatement> {
    match &statement.facts {
        PostgresSqlStatementKind::AlterTable { .. } => Some(statement),
        PostgresSqlStatementKind::DoBlock { block } => {
            block.statements.iter().find_map(first_alter)
        }
        PostgresSqlStatementKind::Conditional { branches } => branches
            .iter()
            .flat_map(|branch| &branch.statements)
            .find_map(first_alter),
        _ => None,
    }
}

#[test]
fn selective_foreign_key_actions_keep_nested_utility_blocks_complete() {
    let sql = fixture("procedural-selective-fk-valid.sql");
    let parsed = facts("procedural-selective-fk-valid.sql");
    assert!(parsed.diagnostics.is_empty(), "{parsed:?}");
    assert_eq!(parsed.statements.len(), 3, "{parsed:?}");
    for (index, statement) in parsed.statements.iter().enumerate() {
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!("expected DO block {index}: {statement:?}");
        };
        assert!(block.complete, "block {index}: {:?}", block.diagnostics);
        assert!(block.diagnostics.is_empty(), "block {index}: {block:?}");
        assert_eq!(
            count_utilities(&block.occurrences),
            if index == 0 { 3 } else { 1 }
        );
        assert_eq!(
            &sql[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
    }
    let PostgresSqlStatementKind::DoBlock { block } = &parsed.statements[0].facts else {
        unreachable!()
    };
    let outer = &block.occurrences[0];
    let nested = &outer.occurrences[2].occurrences[0];
    assert_eq!(nested.kind, Kind::Utility);
    assert!(sql[nested.span.start.offset..nested.span.end.offset]
        .contains("ON DELETE SET NULL (parent_id) NOT VALID"));
    let alter = first_alter(&parsed.statements[0]).expect("nested ALTER TABLE facts");
    let PostgresSqlStatementKind::AlterTable { operations, .. } = &alter.facts else {
        unreachable!()
    };
    let PostgresSqlAlterOperation::AddConstraint {
        constraint,
        not_valid,
    } = &operations[0]
    else {
        panic!("expected added foreign key: {operations:?}");
    };
    assert!(*not_valid);
    assert_eq!(constraint.on_delete.as_deref(), Some("SET NULL"));
    let span = constraint.span.as_ref().expect("constraint source span");
    assert!(sql[span.start.offset..span.end.offset]
        .contains("ON DELETE SET NULL (parent_id) NOT VALID"));
}

#[test]
fn invalid_selective_foreign_key_lists_still_fail_closed() {
    for name in [
        "procedural-selective-fk-invalid-empty.sql",
        "procedural-selective-fk-invalid-expression.sql",
        "procedural-selective-fk-invalid-column.sql",
        "procedural-selective-fk-invalid-duplicate.sql",
        "procedural-selective-fk-invalid-on-update.sql",
    ] {
        let parsed = facts(name);
        assert_eq!(parsed.statements.len(), 1, "{name}: {parsed:?}");
        let PostgresSqlStatementKind::DoBlock { block } = &parsed.statements[0].facts else {
            panic!("expected DO block: {name}: {parsed:?}");
        };
        assert!(!block.complete, "{name}: {block:?}");
        assert!(!block.diagnostics.is_empty(), "{name}: {block:?}");
        assert_eq!(count_utilities(&block.occurrences), 1, "{name}: {block:?}");
    }
}

#[test]
fn selective_utility_does_not_hide_executable_siblings() {
    let parsed = facts("procedural-selective-fk-executable.sql");
    let PostgresSqlStatementKind::DoBlock { block } = &parsed.statements[0].facts else {
        panic!("expected DO block: {parsed:?}");
    };
    assert!(!block.complete, "{block:?}");
    assert_eq!(count_utilities(&block.occurrences), 1);
    assert!(contains_kind(&block.occurrences, Kind::DynamicExecute));
    assert!(contains_kind(&block.occurrences, Kind::Dml));
}
