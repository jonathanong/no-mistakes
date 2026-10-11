use super::*;
use std::collections::BTreeSet;

fn span(start: usize, end: usize) -> PostgresSqlSpan {
    let position = |offset| PostgresSqlPosition {
        offset,
        line: 1,
        column: offset + 1,
    };
    PostgresSqlSpan {
        start: position(start),
        end: position(end),
    }
}

fn statement(facts: PostgresSqlStatementKind, start: usize, end: usize) -> PostgresSqlStatement {
    PostgresSqlStatement {
        ordinal: 0,
        span: span(start, end),
        sql: String::new(),
        facts,
    }
}

fn wrapper(child: PostgresSqlStatement) -> PostgresSqlWrapper {
    PostgresSqlWrapper {
        wrapper_kind: PostgresSqlWrapperKind::Explain,
        execution: PostgresSqlExecution::NonExecuting,
        statements: vec![child],
        span: None,
        complete: false,
        diagnostics: Vec::new(),
    }
}

#[test]
fn classified_utility_spans_propagate_through_wrappers() {
    let child = statement(PostgresSqlStatementKind::Other, 10, 11);
    let spans = BTreeSet::from([(10, 11)]);
    let wrapped = statement(
        PostgresSqlStatementKind::Wrapper {
            wrapper: wrapper(child.clone()),
        },
        0,
        11,
    );
    let function = PostgresSqlFunction {
        wrapper: wrapper(child),
        name: PostgresSqlName {
            parts: Vec::new(),
            sql: String::new(),
        },
        arguments: Vec::new(),
        return_type: None,
        returns_set: false,
        language: None,
        behavior: None,
        body_sql: None,
        or_replace: false,
        temporary: false,
        called_on_null: None,
        parallel: None,
        configuration: Vec::new(),
        security: None,
    };
    let created = statement(PostgresSqlStatementKind::CreateFunction { function }, 0, 11);

    assert!(unsupported(&[wrapped, created], &spans).is_empty());
}
