mod calls;
mod children;
mod functions;
mod members;
mod tagged;
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
        Expression::TaggedTemplateExpression(value) => tagged::tagged(value, expr, source),
        Expression::BinaryExpression(value)
            if value.operator == oxc_ast::ast::BinaryOperator::Addition =>
        {
            Expr::Template(vec![
                expression(&value.left, source),
                expression(&value.right, source),
            ])
        }
        Expression::CallExpression(value) => calls::calls(value, source),
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
                true,
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
        Expression::StaticMemberExpression(value) => Expr::Member(
            Box::new(expression(&value.object, source)),
            value.property.name.to_string(),
        ),
        Expression::ComputedMemberExpression(value) => members::computed(value, source),
        Expression::AwaitExpression(value) => {
            Expr::Await(Box::new(expression(&value.argument, source)))
        }
        Expression::UnaryExpression(value) => {
            if value.operator == oxc_ast::ast::UnaryOperator::Delete {
                members::deleted(&value.argument, source)
            } else {
                Expr::Children(vec![expression(&value.argument, source)])
            }
        }
        Expression::BinaryExpression(value) => Expr::Children(vec![
            expression(&value.left, source),
            expression(&value.right, source),
        ]),
        // Boolean control values carry no SQL text and have no side effects.
        Expression::BooleanLiteral(_) => Expr::Children(vec![]),
        Expression::LogicalExpression(value) => Expr::Children(vec![
            expression(&value.left, source),
            Expr::Alternatives(vec![
                expression(&value.right, source),
                Expr::Children(vec![]),
            ]),
        ]),
        Expression::ConditionalExpression(value) => Expr::Children(vec![
            expression(&value.test, source),
            Expr::Alternatives(vec![
                expression(&value.consequent, source),
                expression(&value.alternate, source),
            ]),
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
