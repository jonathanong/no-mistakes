use super::{summary::Query, ExecutorAccess, Parameter};
use crate::codebase::postgres::embedded::{tags, EmbeddedSqlOptions};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use std::collections::HashSet;

pub(super) fn summarize(
    call: &CallExpression<'_>,
    params: &[&str],
    source: &str,
    bindings: &HashSet<String>,
    options: &EmbeddedSqlOptions,
    tags: &tags::SqlTagNames,
) -> Option<Vec<Query>> {
    let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if params.contains(&callee.name.as_str()) || !bindings.contains(callee.name.as_str()) {
        return None;
    }
    let mut selected = None;
    for argument in call.arguments.iter().skip(1) {
        let Expression::Identifier(id) = unwrap_ts_wrappers(argument.as_expression()?) else {
            continue;
        };
        let index = params.iter().position(|param| *param == id.name.as_str())?;
        if selected.replace(index).is_some() {
            return None;
        }
    }
    let executor = selected?;
    if options.executor_option_properties.is_empty() {
        return None;
    }
    let properties = options.executor_option_properties.clone();
    let expression = unwrap_ts_wrappers(call.arguments.first()?.as_expression()?);
    let arms = match expression {
        Expression::ConditionalExpression(conditional) => {
            let Expression::Identifier(test) = unwrap_ts_wrappers(&conditional.test) else {
                return None;
            };
            let parameter = params
                .iter()
                .position(|param| *param == test.name.as_str())?;
            vec![
                (&conditional.consequent, Some((parameter, true))),
                (&conditional.alternate, Some((parameter, false))),
            ]
        }
        expression => vec![(expression, None)],
    };
    arms.into_iter()
        .map(|(expression, condition)| {
            let expression = unwrap_ts_wrappers(expression);
            let mut parameters = Vec::new();
            match expression {
                Expression::StringLiteral(_) => {}
                Expression::TemplateLiteral(template) if template.expressions.is_empty() => {}
                Expression::TaggedTemplateExpression(tagged) => {
                    if tags::interpolating_untrusted_tag(
                        expression,
                        &mut |name| params.contains(&name),
                        tags,
                    ) {
                        return None;
                    }
                    parameters = tagged
                        .quasi
                        .expressions
                        .iter()
                        .map(|value| super::parameters::binding(value, params))
                        .collect::<Option<Vec<Parameter>>>()?;
                }
                _ => return None,
            }
            if let Some(argument) = call.arguments.get(1) {
                if let Expression::ArrayExpression(array) =
                    unwrap_ts_wrappers(argument.as_expression()?)
                {
                    if !parameters.is_empty() {
                        return None;
                    }
                    parameters = array
                        .elements
                        .iter()
                        .map(|value| super::parameters::binding(value.as_expression()?, params))
                        .collect::<Option<Vec<_>>>()?;
                }
            }
            Some(Query {
                executor,
                option_property: Some(properties.clone()),
                condition,
                access: ExecutorAccess::Object,
                parameters,
                span: (call.span.start, call.span.end),
                sql: positional_sql(expression)?,
                line: crate::codebase::ts_source::byte_offset_to_line(
                    source,
                    call.span.start as usize,
                ),
            })
        })
        .collect()
}

fn positional_sql(expression: &Expression<'_>) -> Option<String> {
    let Expression::TaggedTemplateExpression(tagged) = expression else {
        return crate::codebase::postgres::embedded::sql_text(expression);
    };
    let mut sql = String::new();
    for (index, quasi) in tagged.quasi.quasis.iter().enumerate() {
        if index > 0 {
            sql.push_str(&format!("${index}"));
        }
        sql.push_str(&tags::quasi_text(
            quasi,
            tags::is_string_raw_tag_spelling(&tagged.tag),
        ));
    }
    Some(sql)
}
