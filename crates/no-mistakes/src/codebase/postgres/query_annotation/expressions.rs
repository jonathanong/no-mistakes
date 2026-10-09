use super::{Expr, Function, Step};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ArrowFunctionBody, BindingPattern, Expression, FormalParameters};

pub(super) fn expression(expr: &Expression<'_>, source: &str) -> Expr {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(value) => Expr::Text(value.value.to_string()),
        Expression::Identifier(value) => Expr::Name(value.name.to_string()),
        Expression::TemplateLiteral(value) => {
            let mut parts = Vec::new();
            for (index, quasi) in value.quasis.iter().enumerate() {
                if index > 0 {
                    parts.push(expression(&value.expressions[index - 1], source));
                }
                parts.push(Expr::Text(
                    quasi
                        .value
                        .cooked
                        .as_ref()
                        .expect("successful untagged templates have cooked text")
                        .to_string(),
                ));
            }
            Expr::Template(parts)
        }
        Expression::TaggedTemplateExpression(value) => {
            let tag = match unwrap_ts_wrappers(&value.tag) {
                Expression::Identifier(tag) => tag.name.to_string(),
                Expression::StaticMemberExpression(member)
                    if member.property.name == "raw"
                        && matches!(unwrap_ts_wrappers(&member.object), Expression::Identifier(id) if id.name == "String") =>
                {
                    "String.raw".into()
                }
                _ => return Expr::Unknown,
            };
            let parts = if tag == "String.raw" {
                let mut parts = Vec::new();
                for (index, quasi) in value.quasi.quasis.iter().enumerate() {
                    if index > 0 {
                        parts.push(expression(&value.quasi.expressions[index - 1], source));
                    }
                    parts.push(Expr::Text(quasi.value.raw.to_string()));
                }
                parts
            } else {
                vec![Expr::Text(super::super::sql_text(expr).unwrap_or_default())]
            };
            Expr::Tagged(tag, parts)
        }
        Expression::BinaryExpression(value)
            if value.operator == oxc_ast::ast::BinaryOperator::Addition =>
        {
            Expr::Template(vec![
                expression(&value.left, source),
                expression(&value.right, source),
            ])
        }
        Expression::CallExpression(value) => {
            let args = value
                .arguments
                .iter()
                .map(|arg| {
                    arg.as_expression()
                        .map_or(Expr::Unknown, |arg| expression(arg, source))
                })
                .collect::<Vec<_>>();
            if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&value.callee) {
                if member.property.name == "append" && args.len() == 1 && !value.optional {
                    return Expr::Append(
                        Box::new(expression(&member.object, source)),
                        Box::new(args[0].clone()),
                    );
                }
            }
            Expr::Call {
                callee: Box::new(if value.optional {
                    Expr::Unknown
                } else {
                    expression(&value.callee, source)
                }),
                args,
                start: value.span.start,
            }
        }
        Expression::ArrowFunctionExpression(value) => {
            let body = match &value.body {
                ArrowFunctionBody::FunctionBody(body) => {
                    super::statements::steps(&body.statements, source)
                }
                other => vec![Step::Return(
                    other
                        .as_expression()
                        .map_or(Expr::Unknown, |expr| expression(expr, source)),
                )],
            };
            Expr::Function(function(
                &value.params,
                body,
                true,
                value.r#async,
                value.span.start,
            ))
        }
        Expression::FunctionExpression(value) => {
            let mut function = function_expression(value, source);
            if let (Some(id), Expr::Function(summary)) = (&value.id, &mut function) {
                summary.self_name = Some(id.name.to_string());
            }
            function
        }
        Expression::AwaitExpression(value) => expression(&value.argument, source),
        Expression::ArrayExpression(value) => Expr::Children(
            value
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
                .map(|value| expression(value, source))
                .collect(),
        ),
        _ => Expr::Unknown,
    }
}

pub(super) fn function_expression(value: &oxc_ast::ast::Function<'_>, source: &str) -> Expr {
    let body = value
        .body
        .as_ref()
        .map(|body| super::statements::steps(&body.statements, source))
        .unwrap_or_default();
    Expr::Function(function(
        &value.params,
        body,
        !value.generator,
        value.r#async,
        value.span.start,
    ))
}

fn function(
    params: &FormalParameters<'_>,
    body: Vec<Step>,
    supported: bool,
    asynchronous: bool,
    start: u32,
) -> Function {
    let names = params
        .items
        .iter()
        .map(|param| match &param.pattern {
            BindingPattern::BindingIdentifier(id) => Some(id.name.to_string()),
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    let supported = supported
        && names.is_some()
        && params.rest.is_none()
        && !body.iter().any(|step| matches!(step, Step::Unsupported));
    let names = names.unwrap_or_else(|| {
        params
            .items
            .iter()
            .flat_map(|param| super::trust::bound_names(&param.pattern))
            .collect()
    });
    let mut names = names;
    if let Some(rest) = &params.rest {
        names.extend(super::trust::bound_names(&rest.rest.argument));
    }
    Function {
        start,
        params: names,
        body,
        supported,
        asynchronous,
        self_name: None,
    }
}
