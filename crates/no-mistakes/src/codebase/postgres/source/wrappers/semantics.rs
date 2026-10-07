use super::*;
pub(in crate::codebase::postgres::source) fn finalize(
    facts: &mut PostgresSqlStatementKind,
    span: &PostgresSqlSpan,
) {
    let wrapper = match facts {
        PostgresSqlStatementKind::Wrapper { wrapper } => wrapper,
        PostgresSqlStatementKind::CreateFunction { function } => &mut function.wrapper,
        PostgresSqlStatementKind::Insert { insert } => {
            insert.span = Some(span.clone());
            return;
        }
        _ => return,
    };
    wrapper.span = Some(span.clone());
    for diagnostic in &mut wrapper.diagnostics {
        if diagnostic.span.is_none() {
            diagnostic.span = Some(span.clone());
        }
    }
}

pub(super) fn empty(
    kind: PostgresSqlWrapperKind,
    execution: PostgresSqlExecution,
) -> PostgresSqlWrapper {
    PostgresSqlWrapper {
        wrapper_kind: kind,
        execution,
        statements: Vec::new(),
        span: None,
        complete: false,
        diagnostics: Vec::new(),
    }
}
pub(super) fn unsupported(kind: PostgresSqlWrapperKind, message: &str) -> PostgresSqlWrapper {
    let execution = if kind == PostgresSqlWrapperKind::Prepare {
        PostgresSqlExecution::NonExecuting
    } else {
        PostgresSqlExecution::Unknown
    };
    let mut wrapper = empty(kind, execution);
    wrapper.diagnostics.push(diagnostic(message, None));
    wrapper
}
pub(super) fn diagnostic(message: &str, span: Option<PostgresSqlSpan>) -> PostgresSqlDiagnostic {
    PostgresSqlDiagnostic {
        message: message.into(),
        span,
    }
}
