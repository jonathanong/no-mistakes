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
