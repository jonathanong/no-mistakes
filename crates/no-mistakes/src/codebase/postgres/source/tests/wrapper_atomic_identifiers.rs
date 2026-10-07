use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn atomic_boundary_preserves_identifier_aliases_and_independent_neighbors() {
    let result = facts("wrapper-atomic-identifiers.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 8);
    let sql = fixture("wrapper-atomic-identifiers.sql");
    for (index, expected) in [
        (1, "SELECT 93;"),
        (3, "SELECT 94;"),
        (5, "SELECT 95;"),
        (7, "SELECT 96;"),
    ] {
        assert_eq!(result.statements[index].sql, expected);
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[0].facts else {
        panic!("function expected")
    };
    // Existing query facts explicitly omit IS DISTINCT FROM expression facts;
    // that incompleteness must propagate without changing the body boundary.
    assert!(!function.wrapper.complete);
    assert_eq!(function.wrapper.statements.len(), 15);
    for (index, child) in function.wrapper.statements.iter().enumerate() {
        let PostgresSqlStatementKind::Select { query } = &child.facts else {
            panic!("query expected")
        };
        assert_eq!(query.complete, index != 11, "{}", child.sql);
    }
    for child in &function.wrapper.statements {
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
        assert!(matches!(
            child.facts,
            PostgresSqlStatementKind::Select { .. }
        ));
    }
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[2].facts else {
        panic!("outer function expected")
    };
    assert!(
        function.wrapper.complete,
        "{:?}",
        function.wrapper.diagnostics
    );
    let PostgresSqlStatementKind::CreateFunction { function } =
        &function.wrapper.statements[0].facts
    else {
        panic!("nested function expected")
    };
    assert!(function.wrapper.complete);
    assert_eq!(function.wrapper.statements.len(), 1);
    let PostgresSqlStatementKind::CreateFunction { function } = &result.statements[4].facts else {
        panic!("unsupported function expected")
    };
    assert!(!function.wrapper.complete);
}
