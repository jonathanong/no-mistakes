use super::{summary::Query, ScopeVisitor};
use crate::codebase::postgres::embedded::EmbeddedSqlCall;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn enabled(call: &CallExpression<'_>, query: &Query) -> Option<bool> {
    let Some((index, expected)) = query.condition else {
        return Some(true);
    };
    match unwrap_ts_wrappers(call.arguments.get(index)?.as_expression()?) {
        Expression::BooleanLiteral(value) => Some(value.value == expected),
        _ => None,
    }
}

pub(super) fn annotate(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    query: &Query,
    fact: &mut EmbeddedSqlCall,
) {
    let Some(expression) = call
        .arguments
        .get(query.executor)
        .and_then(|arg| arg.as_expression())
    else {
        return;
    };
    if let Some(property) = &query.option_property {
        if let Some(Some(receiver)) =
            super::super::transaction::options::receiver(visitor, expression, property)
        {
            super::super::transaction::annotate_receiver(visitor, receiver, call.span.start, fact);
        }
    } else {
        let receiver = match unwrap_ts_wrappers(expression) {
            Expression::StaticMemberExpression(member) if member.property.name == "query" => {
                &member.object
            }
            receiver => receiver,
        };
        super::super::transaction::annotate_receiver(visitor, receiver, call.span.start, fact);
    }
}
