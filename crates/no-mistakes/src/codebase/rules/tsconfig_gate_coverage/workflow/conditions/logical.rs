use super::{expression_bool, InputState, StaticBool};

pub(super) fn compound_bool(expression: &str, inputs: &InputState) -> Option<StaticBool> {
    if let Some(index) = top_level_operator(expression, b"||") {
        return Some(or(
            expression_bool(&expression[..index], inputs),
            expression_bool(&expression[index + 2..], inputs),
        ));
    }
    if let Some(index) = top_level_operator(expression, b"&&") {
        return Some(and(
            expression_bool(&expression[..index], inputs),
            expression_bool(&expression[index + 2..], inputs),
        ));
    }
    outer_parentheses_body(expression).map(|body| expression_bool(body, inputs))
}

pub(super) fn comparison_operands(expression: &str) -> Option<(&str, &str, bool)> {
    let equal = top_level_operator(expression, b"==");
    let not_equal = top_level_operator(expression, b"!=");
    let (index, equal) = match (equal, not_equal) {
        (Some(index), None) => (index, true),
        (None, Some(index)) => (index, false),
        _ => return None,
    };
    let right = &expression[index + 2..];
    if top_level_operator(right, b"==").is_some() || top_level_operator(right, b"!=").is_some() {
        return None;
    }
    Some((&expression[..index], right, equal))
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

fn outer_parentheses_body(expression: &str) -> Option<&str> {
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
