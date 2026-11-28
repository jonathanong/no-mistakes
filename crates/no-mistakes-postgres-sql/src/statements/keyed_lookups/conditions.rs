use super::function_clauses::{
    coalesce_arguments, split_coalesce_arguments, strict_null_arguments,
};
use crate::fx::FxHashSet;
use crate::idents::{function_arg_expr, unwrap_expr};
use sqlparser::ast::{BinaryOperator, CaseWhen, Expr, Value};

pub(super) fn literal_condition(expr: &Expr) -> Option<bool> {
    match literal(expr)? {
        Value::Boolean(value) => Some(*value),
        Value::Null => Some(false),
        _ => None,
    }
}

pub(super) fn case_condition(operand: Option<&Expr>, condition: &Expr) -> Option<bool> {
    let Some(operand) = operand else {
        return literal_condition(condition);
    };
    let (left, right) = (literal(operand)?, literal(condition)?);
    match (left, right) {
        (Value::Null, _) | (_, Value::Null) => Some(false),
        (Value::Boolean(left), Value::Boolean(right)) => Some(left == right),
        (Value::Number(left, _), Value::Number(right, _)) => {
            Some(decimal_integer(left)? == decimal_integer(right)?)
        }
        _ => None,
    }
}

pub(super) fn visit_case_children(
    operand: Option<&Expr>,
    conditions: &[CaseWhen],
    else_result: Option<&Expr>,
    mut visit: impl FnMut(&Expr, bool),
) {
    let mut reachable = true;
    for branch in conditions {
        let condition = case_condition(operand, &branch.condition);
        visit(&branch.condition, reachable);
        visit(&branch.result, reachable && condition != Some(false));
        reachable &= condition != Some(true);
    }
    if let Some(else_result) = else_result {
        visit(else_result, reachable);
    }
}

pub(super) fn decimal_integer(number: &str) -> Option<i64> {
    if number.contains('_') {
        number.replace('_', "").parse().ok()
    } else {
        number.parse().ok()
    }
}

pub(super) fn literal(expr: &Expr) -> Option<&Value> {
    match unwrap_expr(expr) {
        Expr::Value(value) => Some(&value.value),
        _ => None,
    }
}

pub(super) fn boolean_result_is_fixed(left: &Expr, op: &BinaryOperator, right: &Expr) -> bool {
    let annihilator = match op {
        BinaryOperator::And => false,
        BinaryOperator::Or => true,
        _ => return false,
    };
    [left, right]
        .into_iter()
        .any(|expr| matches!(literal(expr), Some(Value::Boolean(value)) if *value == annihilator))
}

pub(super) fn cannot_match(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => cannot_match(left) || cannot_match(right),
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Or,
            right,
        } => cannot_match(left) && cannot_match(right),
        _ => literal_condition(expr) == Some(false),
    }
}

pub(super) fn collect_pruned_children(expr: &Expr, roots: &mut FxHashSet<*const Expr>) {
    match expr {
        Expr::BinaryOp { left, op, right } if boolean_result_is_fixed(left, op, right) => {
            roots.extend([left.as_ref(), right.as_ref()].map(std::ptr::from_ref));
        }
        Expr::Case {
            operand,
            conditions,
            else_result,
            ..
        } => {
            visit_case_children(
                operand.as_deref(),
                conditions,
                else_result.as_deref(),
                |child, reachable| {
                    if !reachable {
                        roots.insert(std::ptr::from_ref(child));
                    }
                },
            );
        }
        Expr::Function(function) => {
            let skipped = strict_null_arguments(function).or_else(|| {
                coalesce_arguments(function).map(|args| split_coalesce_arguments(args).1)
            });
            if let Some(skipped) = skipped {
                roots.extend(
                    skipped
                        .iter()
                        .filter_map(function_arg_expr)
                        .map(std::ptr::from_ref),
                );
            }
        }
        _ => {}
    }
}
