use super::{summary::Query, ExecutorAccess};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn summarize(call: &CallExpression<'_>, params: &[&str], source: &str) -> Option<Query> {
    if call.arguments.len() > 2 {
        return None;
    }
    let access = if matches!(
        unwrap_ts_wrappers(&call.callee),
        Expression::StaticMemberExpression(_)
    ) {
        ExecutorAccess::Object
    } else {
        ExecutorAccess::Callable
    };
    let receiver = match unwrap_ts_wrappers(&call.callee) {
        Expression::StaticMemberExpression(member)
            if member.property.name == "query" && !member.optional =>
        {
            unwrap_ts_wrappers(&member.object)
        }
        receiver => receiver,
    };
    let Expression::Identifier(id) = receiver else {
        return None;
    };
    let executor = params.iter().position(|param| *param == id.name.as_str())?;
    let sql = unwrap_ts_wrappers(call.arguments.first()?.as_expression()?);
    if !matches!(sql, Expression::StringLiteral(_))
        && !matches!(sql, Expression::TemplateLiteral(template) if template.expressions.is_empty())
    {
        return None;
    }
    let sql = crate::codebase::postgres::embedded::sql_text(sql)?;
    if let Some(argument) = call.arguments.get(1) {
        let Expression::ArrayExpression(array) = unwrap_ts_wrappers(argument.as_expression()?)
        else {
            return None;
        };
        if array
            .elements
            .iter()
            .any(|element| element.as_expression().is_none())
        {
            return None;
        }
    }
    let parameters = call
        .arguments
        .get(1)
        .and_then(|argument| argument.as_expression())
        .map(unwrap_ts_wrappers)
        .and_then(|expression| match expression {
            Expression::ArrayExpression(array) => Some(
                array
                    .elements
                    .iter()
                    .map(|element| {
                        super::parameters::parameter(element.as_expression().unwrap(), params)
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default();
    Some(super::summary::Query {
        executor,
        access,
        parameters,
        span: (call.span.start, call.span.end),
        sql,
        line: crate::codebase::ts_source::byte_offset_to_line(source, call.span.start as usize),
    })
}
