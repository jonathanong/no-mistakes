use super::StaticBool;
use super::StaticValue;

pub(super) fn hexadecimal_bool(expression: &str) -> Option<StaticBool> {
    let expression = expression.strip_prefix('-').unwrap_or(expression);
    let digits = expression
        .strip_prefix("0x")
        .or_else(|| expression.strip_prefix("0X"))?;
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(if digits.bytes().all(|byte| byte == b'0') {
        StaticBool::False
    } else {
        StaticBool::TruthyNonBoolean
    })
}

pub(super) fn number_bool(value: Option<f64>) -> StaticBool {
    match value {
        Some(0.0) => StaticBool::False,
        Some(_) => StaticBool::TruthyNonBoolean,
        None => StaticBool::Unknown,
    }
}

pub(super) fn quoted_string_bool(expression: &str) -> Option<StaticBool> {
    let body = quoted_string_body(expression)?;
    Some(if body.is_empty() {
        StaticBool::False
    } else {
        StaticBool::TruthyNonBoolean
    })
}

pub(super) fn quoted_string_body(expression: &str) -> Option<&str> {
    let body = expression.strip_prefix('\'')?.strip_suffix('\'')?;
    let bytes = body.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\'' {
            (bytes.get(index + 1) == Some(&b'\'')).then_some(())?;
            index += 2;
        } else {
            index += 1;
        }
    }
    Some(body)
}

pub(super) fn strip_expression(expression: &str) -> &str {
    expression
        .strip_prefix("${{")
        .and_then(|body| body.strip_suffix("}}"))
        .map(str::trim)
        .unwrap_or(expression)
}

pub(super) fn status_function_bool(
    expression: &str,
    status: super::ConditionStatus,
) -> Option<StaticBool> {
    let expression = expression.trim();
    if let Some(operand) = expression.strip_prefix('!') {
        return status_function_bool(operand, status).map(StaticBool::negate);
    }
    if let Some(operand) = expression
        .strip_prefix('(')
        .and_then(|operand| operand.strip_suffix(')'))
    {
        return status_function_bool(operand, status);
    }
    if expression.eq_ignore_ascii_case("success()") {
        Some(status.success)
    } else if expression.eq_ignore_ascii_case("always()") {
        Some(StaticBool::True)
    } else if expression.eq_ignore_ascii_case("failure()") {
        Some(status.failure)
    } else if expression.eq_ignore_ascii_case("cancelled()") {
        Some(StaticBool::False)
    } else {
        None
    }
}

pub(super) fn job_status_value(status: super::ConditionStatus) -> StaticValue {
    match status.success {
        StaticBool::True => StaticValue::String("success".to_string()),
        StaticBool::False => StaticValue::String("failure".to_string()),
        StaticBool::TruthyNonBoolean | StaticBool::Invalid | StaticBool::Unknown => {
            StaticValue::Unknown
        }
    }
}
