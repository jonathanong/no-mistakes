use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod contracts;
mod inputs;
mod literals;
mod logical;

pub(super) use inputs::{callee_inputs, callee_secrets_valid, direct_inputs};
use literals::{
    continues_after_skipped_need, hexadecimal_bool, number_bool, quoted_string_bool,
    strip_expression,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    TruthyNonBoolean,
    Unknown,
}

pub(super) type InputState = BTreeMap<String, StaticBool>;

pub(super) fn statically_skipped_jobs(
    jobs: &serde_yaml::Mapping,
    inputs: &InputState,
    initial_skipped: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut skipped = initial_skipped.clone();
    loop {
        let mut changed = false;
        for (job_id, job) in jobs {
            let job_id = super::normalized_job_id(job_id).expect("validated scalar job ID");
            let directly_disabled = static_bool(job.get("if"), inputs) == StaticBool::False;
            let blocked_by_need = !continues_after_skipped_need(job)
                && crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .iter()
                .any(|need| skipped.contains(&need.to_lowercase()));
            if (directly_disabled || blocked_by_need) && skipped.insert(job_id) {
                changed = true;
            }
        }
        if !changed {
            return skipped;
        }
    }
}

pub(super) fn statically_not_enforcing(value: &Value, inputs: &InputState) -> bool {
    static_bool(value.get("if"), inputs) == StaticBool::False
        || static_bool(value.get("continue-on-error"), inputs) == StaticBool::True
}

fn static_bool(value: Option<&Value>, inputs: &InputState) -> StaticBool {
    match value {
        Some(Value::Bool(value)) => StaticBool::from(*value),
        Some(Value::Number(value)) => number_bool(value.as_f64()),
        Some(Value::Null) => StaticBool::False,
        Some(Value::String(expression)) => expression_bool(expression, inputs),
        _ => StaticBool::Unknown,
    }
}

fn expression_bool(expression: &str, inputs: &InputState) -> StaticBool {
    let expression = strip_expression(expression.trim());
    if super::expressions::condition_expression_valid(expression) {
        if let Some(value) = logical::compound_bool(expression, inputs) {
            return value;
        }
    }
    if expression.is_empty() || expression.eq_ignore_ascii_case("false") {
        return StaticBool::False;
    }
    if expression.eq_ignore_ascii_case("true") {
        return StaticBool::True;
    }
    if expression.eq_ignore_ascii_case("null") {
        return StaticBool::False;
    }
    if let Some(value) = quoted_string_bool(expression) {
        return value;
    }
    if let Some(value) = hexadecimal_bool(expression) {
        return value;
    }
    if let Ok(value) = expression.parse::<f64>() {
        return number_bool(Some(value));
    }
    resolve_input_expression(expression, inputs)
}

fn resolve_input_expression(expression: &str, inputs: &InputState) -> StaticBool {
    for (operator, equal) in [("==", true), ("!=", false)] {
        if let Some((left, right)) = expression.split_once(operator) {
            let (name, expected) = match (input_name(left), bool_literal(right)) {
                (Some(name), Some(expected)) => (name, expected),
                _ => match (bool_literal(left), input_name(right)) {
                    (Some(expected), Some(name)) => (name, expected),
                    _ => return StaticBool::Unknown,
                },
            };
            let value = inputs
                .get(&name.to_lowercase())
                .copied()
                .unwrap_or(StaticBool::False)
                .equals(expected);
            return if equal { value } else { value.negate() };
        }
    }
    if let Some(name) = input_name(expression) {
        return inputs
            .get(&name.to_lowercase())
            .copied()
            .unwrap_or(StaticBool::False)
            .truthiness();
    }
    if let Some(name) = expression
        .strip_prefix('!')
        .map(str::trim)
        .and_then(input_name)
    {
        return inputs
            .get(&name.to_lowercase())
            .copied()
            .unwrap_or(StaticBool::False)
            .truthiness()
            .negate();
    }
    StaticBool::Unknown
}

fn input_name(operand: &str) -> Option<&str> {
    let operand = operand.trim();
    if let Some(name) = operand.strip_prefix("inputs.") {
        let name = name.trim();
        return contracts::valid_identifier(name).then_some(name);
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
    (suffix.trim() == "]" && contracts::valid_identifier(name)).then_some(name)
}

fn bool_literal(operand: &str) -> Option<bool> {
    match operand.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

impl StaticBool {
    fn truthiness(self) -> Self {
        match self {
            Self::TruthyNonBoolean => Self::True,
            value => value,
        }
    }

    fn negate(self) -> Self {
        match self {
            Self::False => Self::True,
            Self::True => Self::False,
            Self::TruthyNonBoolean => Self::False,
            Self::Unknown => Self::Unknown,
        }
    }

    fn equals(self, expected: bool) -> Self {
        if self == Self::TruthyNonBoolean {
            return Self::Unknown;
        }
        if expected {
            self
        } else {
            self.negate()
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

#[cfg(test)]
mod tests;
