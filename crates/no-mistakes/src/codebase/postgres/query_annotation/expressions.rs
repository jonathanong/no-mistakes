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
            Expr::Binary(
                "+".into(),
                Box::new(expression(&value.left, source)),
                Box::new(expression(&value.right, source)),
            )
        }
        Expression::CallExpression(value) => calls::calls(value, source),
        Expression::ImportExpression(value) => {
            crate::codebase::dependencies::extract::static_import_specifier(&value.source)
                .map(Expr::ModuleLoad)
                .unwrap_or_else(|| Expr::Opaque(children::collect(expr, source)))
        }
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
        Expression::NumericLiteral(value)
            if value.value.fract() == 0.0 && value.value.abs() < 9_007_199_254_740_991.0 =>
        {
            Expr::Number(value.value as i64)
        }
        Expression::NumericLiteral(_) => Expr::Primitive,
        Expression::BinaryExpression(value) => Expr::Binary(
            value.operator.as_str().into(),
            Box::new(expression(&value.left, source)),
            Box::new(expression(&value.right, source)),
        ),
        // Boolean control values carry no SQL text and have no side effects.
        Expression::BooleanLiteral(value) => Expr::Boolean(value.value),
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
        Expression::ArrayExpression(array) => Expr::Array(
            array
                .elements
                .iter()
                .map(|element| match element {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        Expr::Spread(Box::new(expression(&spread.argument, source)))
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => Expr::Unknown,
                    other => expression(
                        other.as_expression().expect("array expression element"),
                        source,
                    ),
                })
                .collect(),
        ),
        Expression::ObjectExpression(value) => objects::object(value, source),
        Expression::AssignmentExpression(_) | Expression::UpdateExpression(_) => {
            writes::collect(expr, source)
        }
        _ => Expr::Opaque(children::collect(expr, source)),
    }
}
