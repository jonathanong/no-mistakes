use super::{StaticBool, StaticValue};

pub(super) fn input_name(operand: &str) -> Option<&str> {
    let operand = operand.trim();
    if let Some(name) = operand.strip_prefix("inputs.") {
        let name = name.trim();
        return super::contracts::valid_identifier(name).then_some(name);
    }
    let bracketed = operand
        .strip_prefix("inputs")?
        .trim_start()
        .strip_prefix('[')?
        .trim_start();
    let quote = bracketed.chars().next()?;
    if quote != '\'' {
        return None;
    }
    let name = bracketed.strip_prefix(quote)?;
    let (name, suffix) = name.split_once(quote)?;
    (suffix.trim() == "]" && super::contracts::valid_identifier(name)).then_some(name)
}

impl StaticBool {
    pub(super) fn truthiness(self) -> Self {
        match self {
            Self::TruthyNonBoolean => Self::True,
            value => value,
        }
    }

    pub(super) fn negate(self) -> Self {
        match self {
            Self::False => Self::True,
            Self::True => Self::False,
            Self::TruthyNonBoolean => Self::False,
            Self::Unknown => Self::Unknown,
        }
    }
}

impl From<bool> for StaticBool {
    fn from(value: bool) -> Self {
        if value {
            Self::True
        } else {
            Self::False
        }
    }
}

pub(super) fn comparison_literal(operand: &str) -> Option<StaticValue> {
    let operand = strip_parentheses(operand.trim());
    if operand.eq_ignore_ascii_case("true") {
        return Some(StaticValue::Bool(true));
    }
    if operand.eq_ignore_ascii_case("false") {
        return Some(StaticValue::Bool(false));
    }
    if operand.eq_ignore_ascii_case("null") {
        return Some(StaticValue::Null);
    }
    if let Some(body) = super::literals::quoted_string_body(operand) {
        return Some(StaticValue::String(body.replace("''", "'")));
    }
    expression_number(operand).map(|_| StaticValue::Number(operand.to_string()))
}

impl StaticValue {
    pub(super) fn function_string(&self) -> Option<String> {
        match self {
            Self::Bool(value) => Some(value.to_string()),
            Self::String(value) => Some(value.clone()),
            Self::Number(value) => expression_number(value).map(|value| value.to_string()),
            Self::Null => Some(String::new()),
            Self::Unknown => None,
        }
    }

    pub(super) fn truthiness(self) -> StaticBool {
        match self {
            Self::Bool(value) => StaticBool::from(value),
            Self::String(value) => StaticBool::from(!value.is_empty()),
            Self::Number(value) => expression_number(&value)
                .map(|value| StaticBool::from(value != 0.0))
                .unwrap_or(StaticBool::Unknown),
            Self::Null => StaticBool::False,
            Self::Unknown => StaticBool::Unknown,
        }
    }

    pub(super) fn equals(self, expected: &Self) -> StaticBool {
        if matches!(self, Self::Unknown) || matches!(expected, Self::Unknown) {
            return StaticBool::Unknown;
        }
        match (&self, expected) {
            (Self::String(actual), Self::String(expected)) => {
                return string_equals(actual, expected)
                    .map_or(StaticBool::Unknown, StaticBool::from);
            }
            _ if std::mem::discriminant(&self) == std::mem::discriminant(expected) => {}
            _ => {
                return match (self.loose_number(), expected.clone().loose_number()) {
                    (Some(actual), Some(expected)) => StaticBool::from(actual == expected),
                    _ => StaticBool::False,
                };
            }
        }
        match (self, expected) {
            (Self::Bool(actual), Self::Bool(expected)) => StaticBool::from(actual == *expected),
            (Self::Number(actual), Self::Number(expected)) => {
                match (expression_number(&actual), expression_number(expected)) {
                    (Some(actual), Some(expected)) => StaticBool::from(actual == expected),
                    _ => StaticBool::Unknown,
                }
            }
            (Self::Null, Self::Null) => StaticBool::True,
            _ => StaticBool::Unknown,
        }
    }

    pub(super) fn less_than(self, expected: &Self) -> StaticBool {
        match (self.loose_number(), expected.clone().loose_number()) {
            (Some(actual), Some(expected)) => StaticBool::from(actual < expected),
            _ => StaticBool::Unknown,
        }
    }

    pub(super) fn less_than_or_equal(self, expected: &Self) -> StaticBool {
        match (self.loose_number(), expected.clone().loose_number()) {
            (Some(actual), Some(expected)) => StaticBool::from(actual <= expected),
            _ => StaticBool::Unknown,
        }
    }

    fn loose_number(self) -> Option<f64> {
        match self {
            Self::Bool(value) => Some(f64::from(u8::from(value))),
            Self::Number(value) => expression_number(&value),
            Self::String(value) if value.is_empty() => Some(0.0),
            Self::String(value) => json_number(&value),
            Self::Null => Some(0.0),
            Self::Unknown => None,
        }
    }
}

fn string_equals(left: &str, right: &str) -> Option<bool> {
    if left.is_ascii() && right.is_ascii() {
        Some(left.eq_ignore_ascii_case(right))
    } else if left == right {
        Some(true)
    } else {
        None
    }
}

pub(super) fn expression_number(value: &str) -> Option<f64> {
    let value = strip_parentheses(value.trim());
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |value| (true, value));
    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        let value = u64::from_str_radix(hex, 16).ok()? as f64;
        return Some(if negative { -value } else { value });
    }
    json_number(value)
}

fn json_number(value: &str) -> Option<f64> {
    serde_json::from_str::<serde_json::Number>(value.trim())
        .ok()?
        .as_f64()
}

fn strip_parentheses(mut value: &str) -> &str {
    while value.starts_with('(') && value.ends_with(')') {
        value = value[1..value.len() - 1].trim();
    }
    value
}
