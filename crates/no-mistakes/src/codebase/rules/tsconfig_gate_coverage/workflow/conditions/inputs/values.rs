use super::{JsonScalar, StaticBool, Value};
use crate::codebase::rules::tsconfig_gate_coverage::workflow::expressions::complete_expression_type;

pub(super) fn default_falsy_state(default: Option<&JsonScalar>) -> StaticBool {
    if default.map(json_scalar_is_falsy).unwrap_or(true) {
        StaticBool::False
    } else {
        StaticBool::TruthyNonBoolean
    }
}

pub(super) fn nonboolean_binding_state(value: &Value) -> StaticBool {
    if value
        .as_str()
        .is_some_and(|text| complete_expression_type(text.trim()).is_some())
    {
        StaticBool::Unknown
    } else if yaml_scalar_is_falsy(value) {
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
