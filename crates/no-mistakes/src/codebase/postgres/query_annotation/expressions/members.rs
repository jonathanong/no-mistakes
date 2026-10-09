use super::{expression, Expr};
use crate::codebase::postgres::query_annotation::DeleteKey;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{ComputedMemberExpression, Expression};

pub(super) fn computed(value: &ComputedMemberExpression<'_>, source: &str) -> Expr {
    let index = static_index(&value.expression);
    let object = expression(&value.object, source);
    match (index, unwrap_ts_wrappers(&value.expression)) {
        (Some(index), _) => Expr::Index(Box::new(object), index),
        (None, Expression::StringLiteral(name)) => {
            Expr::Member(Box::new(object), name.value.to_string())
        }
        (None, _) => Expr::Children(vec![object, expression(&value.expression, source)]),
    }
}

pub(super) fn static_index(value: &Expression<'_>) -> Option<usize> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    let value = match unwrap_ts_wrappers(value) {
        Expression::StringLiteral(value) => {
            return value
                .value
                .parse::<usize>()
                .ok()
                .filter(|index| index.to_string() == value.value.as_str());
        }
        Expression::NumericLiteral(value) => value.value,
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                oxc_ast::ast::UnaryOperator::UnaryPlus | oxc_ast::ast::UnaryOperator::UnaryNegation
            ) =>
        {
            let value = static_number(&unary.argument)?;
            if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation {
                -value
            } else {
                value
            }
        }
        _ => return None,
    };
    (value.is_finite()
        && (0.0..=MAX_SAFE_INTEGER).contains(&value)
        && value.fract() == 0.0
        && (value as usize) as f64 == value)
        .then_some(value as usize)
}

fn static_number(value: &Expression<'_>) -> Option<f64> {
    match unwrap_ts_wrappers(value) {
        Expression::NumericLiteral(value) => Some(value.value),
        Expression::UnaryExpression(unary)
            if matches!(
                unary.operator,
                oxc_ast::ast::UnaryOperator::UnaryPlus | oxc_ast::ast::UnaryOperator::UnaryNegation
            ) =>
        {
            let value = static_number(&unary.argument)?;
            Some(
                if unary.operator == oxc_ast::ast::UnaryOperator::UnaryNegation {
                    -value
                } else {
                    value
                },
            )
        }
        _ => None,
    }
}

pub(super) fn deleted(value: &Expression<'_>, source: &str) -> Expr {
    // A delete mutates the receiver, not merely the selected slot value.
    let (values, key) = match unwrap_ts_wrappers(value) {
        Expression::ComputedMemberExpression(value) => (
            vec![
                expression(&value.object, source),
                expression(&value.expression, source),
            ],
            delete_key(&value.expression),
        ),
        Expression::StaticMemberExpression(value) => {
            (vec![expression(&value.object, source)], DeleteKey::Named)
        }
        _ => return Expr::Opaque(vec![expression(value, source)]),
    };
    Expr::Delete(values, key)
}

fn delete_key(value: &Expression<'_>) -> DeleteKey {
    if let Some(index) = static_index(value) {
        return DeleteKey::Index(index);
    }
    match unwrap_ts_wrappers(value) {
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => DeleteKey::Named,
        _ => DeleteKey::Dynamic,
    }
}

#[cfg(test)]
mod tests;
