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
    let parameters = match call.arguments.get(1) {
        None => Vec::new(),
        Some(argument) => {
            let Expression::ArrayExpression(array) = unwrap_ts_wrappers(argument.as_expression()?)
            else {
                return None;
            };
            array
                .elements
                .iter()
                .map(|element| super::parameters::binding(element.as_expression()?, params))
                .collect::<Option<Vec<_>>>()?
        }
    };
    Some(super::summary::Query {
        executor,
        access,
        parameters,
        span: (call.span.start, call.span.end),
        sql,
        line: crate::codebase::ts_source::byte_offset_to_line(source, call.span.start as usize),
    })
}
