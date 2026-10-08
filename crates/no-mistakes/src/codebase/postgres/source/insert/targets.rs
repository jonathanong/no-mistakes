use super::parsing::assignment::Target;
use crate::codebase::postgres::source::{
    expressions::expression, locations::Locations, PostgresSqlAssignmentTarget,
};

pub(super) fn project(target: &Target, locations: &Locations<'_>) -> PostgresSqlAssignmentTarget {
    PostgresSqlAssignmentTarget {
        base: expression(&target.base, locations),
        subscripts: target
            .subscripts
            .iter()
            .map(|(index, span)| {
                let mut projected = expression(index, locations);
                projected.span = locations.span(*span);
                projected
            })
            .collect(),
        span: locations.span(target.span),
    }
}

/// The parser's expression spans omit closing punctuation; token boundaries retain it.
pub(super) fn arbiter(
    expr: &sqlparser::ast::Expr,
    span: sqlparser::tokenizer::Span,
    locations: &Locations<'_>,
) -> crate::codebase::postgres::source::PostgresSqlExpression {
    let mut projected = expression(expr, locations);
    projected.span = locations.span(span);
    if matches!(expr, sqlparser::ast::Expr::Function(_)) {
        // The root function is the first function collected by the AST visitor.
        projected.functions[0].span = projected.span.clone();
    }
    projected
}
