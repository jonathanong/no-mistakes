use super::{source_positions, unwrap_ts_wrappers, Expression, Recovered, ScopeVisitor};
use oxc_span::GetSpan;

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    member: &oxc_ast::ast::StaticMemberExpression<'_>,
    expr: &Expression<'_>,
    line: u32,
) -> Option<Vec<Recovered>> {
    if member.optional || member.property.name != "sql" {
        return None;
    }
    let Expression::Identifier(alias) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    let (values, decision) = visitor
        .finite_cases
        .last()?
        .values(alias.name.as_str(), visitor.function_scopes.len())?;
    let mut recovered = Vec::new();
    for (index, (sql, _)) in values.iter().enumerate() {
        let mut value = Recovered::empty(line);
        value.sql = sql.clone();
        value.origins = vec![expr.span().start; sql.len()];
        value.positions = source_positions::for_expression(
            expr,
            visitor.source,
            expr.span().start as usize,
            line,
        );
        value.choices.push((decision, index as u32));
        value.enumerated = true;
        recovered.push(value);
    }
    super::super::super::super::limit::bounded(recovered)
}
