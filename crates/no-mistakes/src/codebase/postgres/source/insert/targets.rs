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

pub(super) fn value(
    expr: &sqlparser::ast::Expr,
    facts: Option<&super::parsing::assignment::Facts>,
    locations: &Locations<'_>,
) -> crate::codebase::postgres::source::PostgresSqlExpression {
    match facts {
        Some(facts) => arbiter(expr, facts.value_span, locations),
        None => expression(expr, locations),
    }
}

pub(super) fn conflict(
    target: Option<&sqlparser::ast::ConflictTarget>,
    facts: Option<&super::parsing::ConflictFacts>,
    locations: &Locations<'_>,
) -> crate::codebase::postgres::source::PostgresSqlConflictTarget {
    use crate::codebase::postgres::source::{
        expressions::{identifier, name},
        PostgresSqlConflictTarget,
    };
    use sqlparser::ast::ConflictTarget;
    if let Some(facts) = facts.filter(|facts| !facts.expressions.is_empty()) {
        PostgresSqlConflictTarget::Expressions {
            expressions: facts
                .expressions
                .iter()
                .map(|(expr, span)| arbiter(expr, *span, locations))
                .collect(),
        }
    } else {
        match target {
            None => PostgresSqlConflictTarget::Omitted,
            Some(ConflictTarget::Columns(columns)) => PostgresSqlConflictTarget::Columns {
                columns: columns.iter().map(identifier).collect(),
            },
            Some(ConflictTarget::OnConstraint(constraint)) => {
                PostgresSqlConflictTarget::Constraint {
                    name: name(constraint),
                }
            }
        }
    }
}
