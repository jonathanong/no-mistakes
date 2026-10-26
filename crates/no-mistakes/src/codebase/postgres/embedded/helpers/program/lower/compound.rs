//! Lowering of compound expressions (members, literals with children,
//! conditionals, operators), kept apart so `lower` stays a flat dispatch.

use super::expr::lower;
use super::Scope;
use crate::codebase::postgres::embedded::helpers::program::{Entry, Expr};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BinaryOperator, ChainElement, Expression, LogicalOperator, ObjectPropertyKind, PropertyKey,
    PropertyKind,
};

pub(super) fn member_expression(scope: &mut Scope<'_, '_>, node: &Expression<'_>) -> Expr {
    match node {
        Expression::StaticMemberExpression(member) => Expr::Member(
            Box::new(lower(scope, &member.object)),
            member.property.name.to_string(),
        ),
        Expression::ComputedMemberExpression(member) => {
            match unwrap_ts_wrappers(&member.expression) {
                Expression::StringLiteral(key) => Expr::Member(
                    Box::new(lower(scope, &member.object)),
                    crate::ast::js_literal(&key).into_owned(),
                ),
                _ => Expr::Opaque,
            }
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => Expr::Member(
                Box::new(lower(scope, &member.object)),
                member.property.name.to_string(),
            ),
            _ => Expr::Opaque,
        },
        _ => Expr::Opaque,
    }
}

pub(super) fn array_literal(
    scope: &mut Scope<'_, '_>,
    array: &oxc_ast::ast::ArrayExpression<'_>,
) -> Expr {
    let mut elements = Vec::with_capacity(array.elements.len());
    for element in &array.elements {
        let Some(element) = element.as_expression() else {
            return Expr::Opaque;
        };
        elements.push(lower(scope, element));
    }
    Expr::Array(elements)
}

pub(super) fn conditional(
    scope: &mut Scope<'_, '_>,
    branch: &oxc_ast::ast::ConditionalExpression<'_>,
) -> Expr {
    Expr::Cond(
        Box::new(lower(scope, &branch.test)),
        Box::new(lower(scope, &branch.consequent)),
        Box::new(lower(scope, &branch.alternate)),
    )
}

pub(super) fn logical(
    scope: &mut Scope<'_, '_>,
    logical: &oxc_ast::ast::LogicalExpression<'_>,
) -> Expr {
    let left = Box::new(lower(scope, &logical.left));
    let right = Box::new(lower(scope, &logical.right));
    match logical.operator {
        LogicalOperator::And => Expr::And(left, right),
        LogicalOperator::Or => Expr::Or(left, right),
        LogicalOperator::Coalesce => Expr::Nullish(left, right),
    }
}

pub(super) fn binary_expression(
    scope: &mut Scope<'_, '_>,
    binary: &oxc_ast::ast::BinaryExpression<'_>,
) -> Expr {
    let (strict, negate) = match binary.operator {
        BinaryOperator::StrictEquality => (true, false),
        BinaryOperator::StrictInequality => (true, true),
        BinaryOperator::Equality => (false, false),
        BinaryOperator::Inequality => (false, true),
        BinaryOperator::Addition => {
            return Expr::Add(
                Box::new(lower(scope, &binary.left)),
                Box::new(lower(scope, &binary.right)),
            )
        }
        _ => return Expr::Opaque,
    };
    Expr::Eq(
        Box::new(lower(scope, &binary.left)),
        Box::new(lower(scope, &binary.right)),
        strict,
        negate,
    )
}

pub(super) fn object_literal(
    scope: &mut Scope<'_, '_>,
    object: &oxc_ast::ast::ObjectExpression<'_>,
) -> Expr {
    let mut entries = Vec::with_capacity(object.properties.len());
    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                entries.push(Entry::Spread(lower(scope, &spread.argument)));
            }
            ObjectPropertyKind::ObjectProperty(property) => {
                let key = match &property.key {
                    PropertyKey::StaticIdentifier(ident) if !property.computed => {
                        ident.name.to_string()
                    }
                    PropertyKey::StringLiteral(literal) => {
                        crate::ast::js_literal(&literal).into_owned()
                    }
                    _ => return Expr::Opaque,
                };
                if property.kind != PropertyKind::Init || property.method {
                    return Expr::Opaque;
                }
                entries.push(Entry::Prop(key, lower(scope, &property.value)));
            }
        }
    }
    Expr::Object(entries)
}
