mod calls;
mod children;
mod functions;
mod members;
mod objects;
mod tagged;
mod writes;
use super::Expr;
use crate::codebase::ts_source::unwrap_ts_wrappers;
pub(super) use functions::function_expression;
use oxc_ast::ast::Expression;

pub(super) fn expression(expr: &Expression<'_>, source: &str) -> Expr {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(value) => Expr::Text(value.value.to_string()),
        Expression::Identifier(value) => Expr::Name(value.name.to_string()),
        Expression::ThisExpression(_) => Expr::Name("this".to_string()),
        Expression::TemplateLiteral(value) => tagged::template(value, source),
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
        Expression::ArrowFunctionExpression(value) => functions::arrow(value, source),
        Expression::FunctionExpression(value) => {
            functions::named_function_expression(value, source)
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
            } else if matches!(
                value.operator,
                oxc_ast::ast::UnaryOperator::UnaryPlus
                    | oxc_ast::ast::UnaryOperator::UnaryNegation
                    | oxc_ast::ast::UnaryOperator::BitwiseNot
            ) {
                // Numeric coercion can invoke user-defined conversion hooks.
                // Preserve operand effects, but do not treat the result like
                // a harmless discarded primitive.
                Expr::Opaque(vec![expression(&value.argument, source)])
            } else {
                Expr::Discard(Box::new(expression(&value.argument, source)))
            }
        }
        Expression::NumericLiteral(_) => Expr::Primitive,
        Expression::BinaryExpression(value) => Expr::Children(vec![
            expression(&value.left, source),
            expression(&value.right, source),
        ]),
        // Boolean control values carry no SQL text and have no side effects.
        Expression::BooleanLiteral(_) => Expr::Primitive,
        Expression::LogicalExpression(value) => Expr::Container(vec![
            expression(&value.left, source),
            Expr::Alternatives(vec![
                expression(&value.right, source),
                Expr::Children(vec![]),
            ]),
        ]),
        Expression::ConditionalExpression(value) => Expr::Sequence(vec![
            Expr::Discard(Box::new(expression(&value.test, source))),
            Expr::Alternatives(vec![
                expression(&value.consequent, source),
                expression(&value.alternate, source),
            ]),
        ]),
        Expression::SequenceExpression(value) => Expr::Sequence(
            value
                .expressions
                .iter()
                .map(|expr| expression(expr, source))
                .collect(),
        ),
        Expression::ArrayExpression(_) => Expr::Container(children::collect(expr, source)),
        Expression::ObjectExpression(value) => objects::object(value, source),
        Expression::AssignmentExpression(_) | Expression::UpdateExpression(_) => {
            writes::collect(expr, source)
        }
        _ => Expr::Opaque(children::collect(expr, source)),
    }
}
