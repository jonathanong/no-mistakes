use super::super::PostgresSqlStatementKind;
use super::{facts, fixture};

#[test]
fn nested_conditional_locks_report_source_boundary_gaps_without_panicking() {
    for name in [
        "nested-conditional-locks.sql",
        "conditional-adjacent-locks.sql",
    ] {
        let sql = fixture(name);
        let result = facts(name);
        assert!(result.diagnostics.is_empty());
        assert_eq!(result.statements.len(), 1);
        let statement = &result.statements[0];
        assert_eq!(
            &sql[statement.span.start.offset..statement.span.end.offset],
            statement.sql
        );
        let PostgresSqlStatementKind::DoBlock { block } = &statement.facts else {
            panic!("expected a DO fact")
        };
        assert!(!block.complete);
        assert_eq!(block.diagnostics.len(), 1);
        assert!(block.diagnostics[0]
            .message
            .contains("Conditional statement source range is unavailable"));
        assert!(block.diagnostics[0].span.is_some());
        assert!(block.statements.is_empty());
    }
}

#[test]
fn conditional_boundary_diagnostics_preserve_neighboring_index_facts() {
    let result = facts("nested-conditional-locks-mixed.sql");
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.statements.len(), 3);
    assert!(matches!(
        result.statements[0].facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    ));
    assert!(matches!(
        result.statements[2].facts,
        PostgresSqlStatementKind::CreateIndex { .. }
    ));
    let PostgresSqlStatementKind::DoBlock { block } = &result.statements[1].facts else {
        panic!("expected the middle DO fact")
    };
    assert!(!block.complete);
    assert_eq!(block.diagnostics.len(), 1);
}
