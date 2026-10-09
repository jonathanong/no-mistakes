use super::{expression, Expr};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ComputedMemberExpression, Expression};

pub(super) fn computed(value: &ComputedMemberExpression<'_>, source: &str) -> Expr {
    let index = match unwrap_ts_wrappers(&value.expression) {
        Expression::NumericLiteral(value) if value.value.fract() == 0.0 => {
            Some(value.value as usize)
        }
        Expression::StringLiteral(value) => value
            .value
            .parse::<usize>()
            .ok()
            .filter(|index| index.to_string() == value.value.as_str()),
        _ => None,
    };
    let object = expression(&value.object, source);
    match index {
        Some(index) => Expr::Index(Box::new(object), index),
        None => Expr::Children(vec![object, expression(&value.expression, source)]),
    }
}

pub(super) fn deleted(value: &Expression<'_>, source: &str) -> Expr {
    // A delete mutates the receiver, not merely the selected slot value.
    let values = match unwrap_ts_wrappers(value) {
        Expression::ComputedMemberExpression(value) => vec![
            expression(&value.object, source),
            expression(&value.expression, source),
        ],
        Expression::StaticMemberExpression(value) => vec![expression(&value.object, source)],
        _ => vec![expression(value, source)],
    };
    Expr::Opaque(values)
}
