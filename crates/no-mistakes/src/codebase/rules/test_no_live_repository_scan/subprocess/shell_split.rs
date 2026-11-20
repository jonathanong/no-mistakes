use super::{Invocation, OptionsSlot, Token};
use crate::codebase::ts_source::module_facts::arguments::ArgumentValue;
use serde_json::Value;

pub(super) fn flag_true(values: &[ArgumentValue]) -> bool {
    values.iter().skip(1).any(|value| match value {
        ArgumentValue::Object(fields) => matches!(
            fields.get("shell"),
            Some(ArgumentValue::Scalar(Value::Bool(true)))
        ),
        _ => false,
    })
}

pub(super) fn split(values: &[ArgumentValue]) -> Invocation {
    let Some(ArgumentValue::Scalar(Value::String(text))) = values.first() else {
        return Invocation {
            executable: Token::Unknown,
            argv: Vec::new(),
            options: options(values),
        };
    };
    let mut parts = text.split_whitespace().map(known);
    Invocation {
        executable: parts.next().unwrap_or(Token::Unknown),
        argv: parts.collect(),
        options: options(values),
    }
}

fn known(part: &str) -> Token {
    Token::Known(part.to_string())
}

fn options(values: &[ArgumentValue]) -> OptionsSlot {
    match values.get(1) {
        None => OptionsSlot::Absent,
        Some(ArgumentValue::Object(_)) => OptionsSlot::Object(1),
        Some(_) => OptionsSlot::Unknown,
    }
}
