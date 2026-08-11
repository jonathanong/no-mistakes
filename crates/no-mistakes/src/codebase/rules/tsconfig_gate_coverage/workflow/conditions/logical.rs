use super::{
    expression_bool_with_status_and_environment, ConditionStatus, EnvironmentState, InputState,
    StaticBool,
};

#[derive(Clone, Copy)]
pub(super) enum Comparison {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Clone, Copy)]
pub(super) enum LogicalOperator {
    And,
    Or,
}

pub(super) fn compound_bool(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> Option<StaticBool> {
    if let Some((left, right, LogicalOperator::Or)) = logical_operands(expression) {
        return Some(or(
            expression_bool_with_status_and_environment(left, inputs, environment, status),
            expression_bool_with_status_and_environment(right, inputs, environment, status),
        ));
    }
    if let Some((left, right, LogicalOperator::And)) = logical_operands(expression) {
        return Some(and(
            expression_bool_with_status_and_environment(left, inputs, environment, status),
            expression_bool_with_status_and_environment(right, inputs, environment, status),
        ));
    }
    outer_parentheses_body(expression)
        .map(|body| expression_bool_with_status_and_environment(body, inputs, environment, status))
}

pub(super) fn logical_operands(expression: &str) -> Option<(&str, &str, LogicalOperator)> {
    if let Some(index) = top_level_operator(expression, b"||") {
        return Some((
            &expression[..index],
            &expression[index + 2..],
            LogicalOperator::Or,
        ));
    }
    let index = top_level_operator(expression, b"&&")?;
    Some((
        &expression[..index],
        &expression[index + 2..],
        LogicalOperator::And,
    ))
}

pub(super) fn comparison_operands(expression: &str) -> Option<(&str, &str, Comparison)> {
    let (index, width, comparison) = top_level_comparison(expression)?;
    let right = &expression[index + width..];
    if top_level_comparison(right).is_some() {
        return None;
    }
    Some((&expression[..index], right, comparison))
}

fn and(left: StaticBool, right: StaticBool) -> StaticBool {
    match (left.truthiness(), right.truthiness()) {
        (StaticBool::False, _) | (_, StaticBool::False) => StaticBool::False,
        (StaticBool::True, StaticBool::True) => StaticBool::True,
        _ => StaticBool::Unknown,
    }
}

fn or(left: StaticBool, right: StaticBool) -> StaticBool {
    match (left.truthiness(), right.truthiness()) {
        (StaticBool::True, _) | (_, StaticBool::True) => StaticBool::True,
        (StaticBool::False, StaticBool::False) => StaticBool::False,
        _ => StaticBool::Unknown,
    }
}

fn top_level_operator(expression: &str, operator: &[u8; 2]) -> Option<usize> {
    let bytes = expression.as_bytes();
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 2,
            b'\'' => {
                in_string = !in_string;
                index += 1;
            }
            b'(' | b'[' if !in_string => {
                depth += 1;
                index += 1;
            }
            b')' | b']' if !in_string => {
                depth = depth.saturating_sub(1);
                index += 1;
            }
            _ if !in_string && depth == 0 && bytes.get(index..index + 2) == Some(operator) => {
                return Some(index);
            }
            _ => index += 1,
        }
    }
    None
}

fn top_level_comparison(expression: &str) -> Option<(usize, usize, Comparison)> {
    let bytes = expression.as_bytes();
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 2,
            b'\'' => {
                in_string = !in_string;
                index += 1;
            }
            b'(' | b'[' if !in_string => {
                depth += 1;
                index += 1;
            }
            b')' | b']' if !in_string => {
                depth = depth.saturating_sub(1);
                index += 1;
            }
            b'=' if !in_string && depth == 0 && bytes.get(index + 1) == Some(&b'=') => {
                return Some((index, 2, Comparison::Equal));
            }
            b'!' if !in_string && depth == 0 && bytes.get(index + 1) == Some(&b'=') => {
                return Some((index, 2, Comparison::NotEqual));
            }
            b'<' if !in_string && depth == 0 => {
                let equal = bytes.get(index + 1) == Some(&b'=');
                return Some((
                    index,
                    1 + usize::from(equal),
                    if equal {
                        Comparison::LessThanOrEqual
                    } else {
                        Comparison::LessThan
                    },
                ));
            }
            b'>' if !in_string && depth == 0 => {
                let equal = bytes.get(index + 1) == Some(&b'=');
                return Some((
                    index,
                    1 + usize::from(equal),
                    if equal {
                        Comparison::GreaterThanOrEqual
                    } else {
                        Comparison::GreaterThan
                    },
                ));
            }
            _ => index += 1,
        }
    }
    None
}

pub(super) fn outer_parentheses_body(expression: &str) -> Option<&str> {
    let body = expression.strip_prefix('(')?.strip_suffix(')')?;
    top_level_closing_parenthesis(expression)
        .is_some_and(|index| index + 1 == expression.len())
        .then_some(body.trim())
}

fn top_level_closing_parenthesis(expression: &str) -> Option<usize> {
    let bytes = expression.as_bytes();
    let mut depth = 0;
    let mut in_string = false;
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' if in_string && bytes.get(index + 1) == Some(&b'\'') => index += 2,
            b'\'' => {
                in_string = !in_string;
                index += 1;
            }
            b'(' if !in_string => {
                depth += 1;
                index += 1;
            }
            b')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    None
}
