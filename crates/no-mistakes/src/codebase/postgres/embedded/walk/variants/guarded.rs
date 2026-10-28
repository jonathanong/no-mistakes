//! Recovered SQL suffixes from an exact dominating Set membership guard.
use super::{super::ScopeVisitor, Recovered};
use crate::codebase::postgres::embedded::source_positions;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{Expression, IdentifierReference};
use oxc_span::GetSpan;

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    expression: &Expression<'_>,
    id: &IdentifierReference<'_>,
) -> Option<Vec<Recovered>> {
    let (values, decision) = visitor.guarded_cases.last()?.values(
        id.name.as_str(),
        visitor.function_scopes.len(),
        id.span.start,
    )?;
    let line = byte_offset_to_line(visitor.source, expression.span().start as usize);
    let positions = source_positions::for_expression(
        expression,
        visitor.source,
        expression.span().start as usize,
        line,
    );
    let mut recovered = Vec::with_capacity(values.len());
    for (index, sql) in values.iter().enumerate() {
        let mut value = Recovered::empty(line);
        value.sql = sql.clone();
        value.origins = vec![id.span.start; sql.len()];
        value.positions = positions.clone();
        value.choices.push((decision, index as u32));
        value.enumerated = true;
        recovered.push(value);
    }
    Some(recovered)
}
