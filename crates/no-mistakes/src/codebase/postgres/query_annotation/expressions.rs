mod children;
mod functions;
use super::{Expr, Step};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use functions::function;
pub(super) use functions::function_expression;
use oxc_ast::ast::{ArrowFunctionBody, Expression};

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
                _ => String::new(),
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
            let mut effects = vec![expression(&value.tag, source)];
            if tag != "String.raw" {
                effects.extend(
                    value
                        .quasi
                        .expressions
                        .iter()
                        .map(|expr| expression(expr, source)),
                );
            }
            Expr::Tagged(tag, parts, effects)
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
                    if let oxc_ast::ast::Argument::SpreadElement(spread) = arg {
                        Expr::Spread(Box::new(expression(&spread.argument, source)))
                    } else {
                        expression(
                            arg.as_expression()
                                .expect("non-spread arguments are expressions"),
                            source,
                        )
                    }
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
                    Expr::Children(vec![expression(&value.callee, source)])
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
        Expression::StaticMemberExpression(value) => {
            Expr::Children(vec![expression(&value.object, source)])
        }
        Expression::ComputedMemberExpression(value) => Expr::Children(vec![
            expression(&value.object, source),
            expression(&value.expression, source),
        ]),
        Expression::AwaitExpression(value) => {
            Expr::Await(Box::new(expression(&value.argument, source)))
        }
        Expression::UnaryExpression(value) => {
            let values = vec![expression(&value.argument, source)];
            if value.operator == oxc_ast::ast::UnaryOperator::Delete {
                Expr::Opaque(values)
            } else {
                Expr::Children(values)
            }
        }
        Expression::BinaryExpression(value) => Expr::Children(vec![
            expression(&value.left, source),
            expression(&value.right, source),
        ]),
        Expression::LogicalExpression(value) => Expr::Children(vec![
            expression(&value.left, source),
            expression(&value.right, source),
        ]),
        Expression::ConditionalExpression(value) => Expr::Children(vec![
            expression(&value.test, source),
            expression(&value.consequent, source),
            expression(&value.alternate, source),
        ]),
        Expression::SequenceExpression(value) => Expr::Children(
            value
                .expressions
                .iter()
                .map(|expr| expression(expr, source))
                .collect(),
        ),
        Expression::ArrayExpression(_) | Expression::ObjectExpression(_) => {
            Expr::Children(children::collect(expr, source))
        }
        _ => Expr::Opaque(children::collect(expr, source)),
    }
}
