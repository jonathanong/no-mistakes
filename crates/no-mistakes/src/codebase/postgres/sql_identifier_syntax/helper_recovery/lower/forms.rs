use super::{Expr, Scope};
use crate::codebase::postgres::embedded::helpers::program::Entry;
use oxc_ast::ast::{
    ArrayExpressionElement, BinaryOperator, Expression, LogicalOperator, ObjectPropertyKind,
    PropertyKey,
};

pub(super) fn lower(scope: &mut Scope<'_>, node: &Expression<'_>) -> Expr {
    match node {
        Expression::ObjectExpression(object) => {
            let mut props = Vec::new();
            for item in &object.properties {
                let ObjectPropertyKind::ObjectProperty(prop) = item else {
                    return Expr::Opaque;
                };
                if prop.method || prop.computed || prop.kind != oxc_ast::ast::PropertyKind::Init {
                    return Expr::Opaque;
                }
                let key = match &prop.key {
                    PropertyKey::StaticIdentifier(key) => key.name.to_string(),
                    PropertyKey::StringLiteral(key) => key.value.to_string(),
                    _ => return Expr::Opaque,
                };
                // Special prototype setters do not form plain own data properties.
                if key == "__proto__" {
                    return Expr::Opaque;
                }
                props.push(Entry::Prop(key, scope.expression(&prop.value)));
            }
            Expr::Object(props)
        }
        Expression::ArrayExpression(array) => {
            let mut items = Vec::new();
            for item in &array.elements {
                if matches!(
                    item,
                    ArrayExpressionElement::SpreadElement(_) | ArrayExpressionElement::Elision(_)
                ) {
                    return Expr::Opaque;
                }
                let Some(item) = item.as_expression() else {
                    return Expr::Opaque;
                };
                items.push(scope.expression(item));
            }
            Expr::Array(items)
        }
        Expression::ConditionalExpression(value) => Expr::Cond(
            Box::new(scope.expression(&value.test)),
            Box::new(scope.expression(&value.consequent)),
            Box::new(scope.expression(&value.alternate)),
        ),
        Expression::LogicalExpression(value) => {
            let a = Box::new(scope.expression(&value.left));
            let b = Box::new(scope.expression(&value.right));
            match value.operator {
                LogicalOperator::And => Expr::And(a, b),
                LogicalOperator::Or => Expr::Or(a, b),
                LogicalOperator::Coalesce => Expr::Nullish(a, b),
            }
        }
        Expression::BinaryExpression(value) => {
            let a = Box::new(scope.expression(&value.left));
            let b = Box::new(scope.expression(&value.right));
            match value.operator {
                BinaryOperator::Addition => Expr::Add(a, b),
                BinaryOperator::Equality => Expr::Eq(a, b, false, false),
                BinaryOperator::Inequality => Expr::Eq(a, b, false, true),
                BinaryOperator::StrictEquality => Expr::Eq(a, b, true, false),
                BinaryOperator::StrictInequality => Expr::Eq(a, b, true, true),
                _ => Expr::Opaque,
            }
        }
        _ => Expr::Opaque,
    }
}
