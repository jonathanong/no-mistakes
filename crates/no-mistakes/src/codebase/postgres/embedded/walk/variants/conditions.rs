use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, SwitchStatement};

pub(super) fn truth(expr: &Expression<'_>) -> Option<bool> {
    let mut expression = unwrap_ts_wrappers(expr);
    let mut invert = false;
    while let Expression::UnaryExpression(unary) = expression {
        if unary.operator != oxc_ast::ast::UnaryOperator::LogicalNot {
            break;
        }
        invert = !invert;
        expression = unwrap_ts_wrappers(&unary.argument);
    }
    let truth = match expression {
        Expression::BooleanLiteral(value) => value.value,
        Expression::NullLiteral(_) => false,
        Expression::NumericLiteral(value) => value.value != 0.0 && !value.value.is_nan(),
        Expression::StringLiteral(value) => !value.value.is_empty(),
        _ => return None,
    };
    Some(truth != invert)
}

#[derive(PartialEq)]
enum Constant<'a> {
    Null,
    Boolean(bool),
    Number(f64),
    Text(&'a str),
}
fn constant<'a>(expression: &'a Expression<'a>) -> Option<Constant<'a>> {
    match unwrap_ts_wrappers(expression) {
        Expression::NullLiteral(_) => Some(Constant::Null),
        Expression::BooleanLiteral(value) => Some(Constant::Boolean(value.value)),
        Expression::NumericLiteral(value) => Some(Constant::Number(value.value)),
        Expression::StringLiteral(value) => Some(Constant::Text(value.value.as_str())),
        _ => None,
    }
}
pub(super) fn selected_case(statement: &SwitchStatement<'_>) -> Option<u32> {
    let discriminant = constant(&statement.discriminant)?;
    let mut default = statement.cases.len() as u32;
    for (index, case) in statement.cases.iter().enumerate() {
        match &case.test {
            Some(test) => {
                if constant(test)? == discriminant {
                    return Some(index as u32);
                }
            }
            None => default = index as u32,
        }
    }
    Some(default)
}

pub(super) fn choice_id(
    visitor: &super::super::ScopeVisitor<'_>,
    expr: &Expression<'_>,
    fallback: u32,
) -> (u64, u32) {
    let mut expression = unwrap_ts_wrappers(expr);
    let mut arm = 0;
    while let Expression::UnaryExpression(unary) = expression {
        if unary.operator != oxc_ast::ast::UnaryOperator::LogicalNot {
            break;
        }
        arm = 1 - arm;
        expression = unwrap_ts_wrappers(&unary.argument);
    }
    if let Expression::Identifier(id) = expression {
        if let Some(key) = visitor
            .lookup(id.name.as_str())
            .and_then(|binding| binding.condition_key)
        {
            return (key, arm);
        }
    }
    (u64::from(fallback), arm)
}

pub(super) fn truth_at(
    visitor: &super::super::ScopeVisitor<'_>,
    expr: &Expression<'_>,
) -> Option<bool> {
    if let Some(truth) = truth(expr) {
        return Some(truth);
    }
    let mut expression = unwrap_ts_wrappers(expr);
    let mut invert = false;
    while let Expression::UnaryExpression(unary) = expression {
        if unary.operator != oxc_ast::ast::UnaryOperator::LogicalNot {
            break;
        }
        invert = !invert;
        expression = unwrap_ts_wrappers(&unary.argument);
    }
    let values = visitor.recover_variants(expression)?;
    let truth = values.first()?.truth();
    values
        .iter()
        .all(|value| value.truth() == truth)
        .then_some(truth != invert)
}
pub(super) fn nullish_at(
    visitor: &super::super::ScopeVisitor<'_>,
    expr: &Expression<'_>,
) -> Option<bool> {
    if truth(expr).is_some() {
        return Some(matches!(
            unwrap_ts_wrappers(expr),
            Expression::NullLiteral(_)
        ));
    }
    let values = visitor.recover_variants(expr)?;
    let nullish = values.first()?.value == super::ValueKind::Null;
    values
        .iter()
        .all(|value| (value.value == super::ValueKind::Null) == nullish)
        .then_some(nullish)
}
