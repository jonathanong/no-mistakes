use super::{facts, fixture};
use crate::codebase::postgres::source::*;

#[test]
fn wrapper_string_options_reuse_decoded_literals_and_preserve_case() {
    let result = facts("wrapper-string-options.sql");
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.statements.len(), 19);
    let sql = fixture("wrapper-string-options.sql");
    for (index, statement) in result.statements.iter().enumerate() {
        let PostgresSqlStatementKind::Wrapper { wrapper } = &statement.facts else {
            panic!("EXPLAIN wrapper expected")
        };
        let expected = match index {
            0..=2 | 18 => PostgresSqlExecution::NonExecuting,
            3..=8 => PostgresSqlExecution::ExecutesForAnalysis,
            _ => PostgresSqlExecution::Unknown,
        };
        assert_eq!(wrapper.execution, expected, "{}", statement.sql);
        assert_eq!(
            wrapper.complete,
            expected != PostgresSqlExecution::Unknown,
            "{}",
            statement.sql
        );
        // Decoded option values must not replace the report's original spelling.
        assert_eq!(
            statement.sql,
            sql[statement.span.start.offset..statement.span.end.offset]
        );
        let child = &wrapper.statements[0];
        assert_eq!(
            child.sql,
            sql[child.span.start.offset..child.span.end.offset]
        );
    }
}
