use super::{JsonScalar, StaticBool, Value};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::{
    complete_expression_type, StaticExpressionType,
};

pub(super) fn default_falsy_state(default: Option<&JsonScalar>) -> StaticBool {
    if default.map(json_scalar_is_falsy).unwrap_or(true) {
        StaticBool::False
    } else {
        StaticBool::TruthyNonBoolean
    }
}

pub(super) fn nonboolean_binding_state(value: &Value) -> StaticBool {
    if let Some(expression) = value.as_str().and_then(static_expression_truthiness) {
        expression
    } else if yaml_scalar_is_falsy(value) {
        StaticBool::False
    } else {
        StaticBool::TruthyNonBoolean
    }
}

fn static_expression_truthiness(text: &str) -> Option<StaticBool> {
    let expression_type = complete_expression_type(text.trim())?;
    let mut body = text.trim().strip_prefix("${{")?.strip_suffix("}}")?.trim();
    while body.starts_with('(') && body.ends_with(')') {
        body = body[1..body.len() - 1].trim();
    }
    match expression_type {
        StaticExpressionType::Dynamic => Some(StaticBool::Unknown),
        StaticExpressionType::Null => Some(StaticBool::False),
        StaticExpressionType::Boolean => match body {
            "false" => Some(StaticBool::False),
            "true" => Some(StaticBool::TruthyNonBoolean),
            _ => Some(StaticBool::Unknown),
        },
        StaticExpressionType::String => body.strip_prefix('\'')?.strip_suffix('\'').map(|value| {
            if value.is_empty() {
                StaticBool::False
            } else {
                StaticBool::TruthyNonBoolean
            }
        }),
        StaticExpressionType::Number => Some(number_truthiness(body)),
    }
}

fn number_truthiness(value: &str) -> StaticBool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        return if hex.bytes().all(|byte| byte == b'0') {
            StaticBool::False
        } else {
            StaticBool::TruthyNonBoolean
        };
    }
    if value.parse::<f64>() == Ok(0.0) {
        StaticBool::False
    } else {
        StaticBool::TruthyNonBoolean
    }
}

fn json_scalar_is_falsy(value: &JsonScalar) -> bool {
    match value {
        JsonScalar::Bool(value) => !value,
        JsonScalar::Number(value) => value.as_f64() == Some(0.0),
        JsonScalar::Text(value) => value.is_empty(),
    }
}

fn yaml_scalar_is_falsy(value: &Value) -> bool {
    value.as_str().is_some_and(str::is_empty)
        || value.as_f64() == Some(0.0)
        || value.as_bool() == Some(false)
}
