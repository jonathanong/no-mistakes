use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

mod inputs;

pub(super) use inputs::{callee_inputs, callee_secrets_valid, direct_inputs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    Unknown,
}

pub(super) type InputState = BTreeMap<String, StaticBool>;

pub(super) fn statically_skipped_jobs(
    jobs: &serde_yaml::Mapping,
    inputs: &InputState,
) -> BTreeSet<String> {
    let mut skipped = BTreeSet::new();
    loop {
        let mut changed = false;
        for (job_id, job) in jobs {
            let Some(job_id) = job_id.as_str() else {
                continue;
            };
            let directly_disabled = static_bool(job.get("if"), inputs) == StaticBool::False;
            let blocked_by_need = !continues_after_skipped_need(job)
                && crate::codebase::workflow_topology::value_primitives::string_list(
                    job.get("needs"),
                )
                .iter()
                .any(|need| skipped.contains(need));
            if (directly_disabled || blocked_by_need) && skipped.insert(job_id.to_string()) {
                changed = true;
            }
        }
        if !changed {
            return skipped;
        }
    }
}

fn continues_after_skipped_need(job: &Value) -> bool {
    job.get("if")
        .and_then(Value::as_str)
        .is_some_and(|expression| {
            matches!(
                strip_expression(expression.trim()),
                "always()" | "!cancelled()"
            )
        })
}

pub(super) fn statically_not_enforcing(value: &Value, inputs: &InputState) -> bool {
    static_bool(value.get("if"), inputs) == StaticBool::False
        || static_bool(value.get("continue-on-error"), inputs) == StaticBool::True
}

fn static_bool(value: Option<&Value>, inputs: &InputState) -> StaticBool {
    match value {
        Some(Value::Bool(value)) => StaticBool::from(*value),
        Some(Value::String(expression)) => expression_bool(expression, inputs),
        _ => StaticBool::Unknown,
    }
}

fn expression_bool(expression: &str, inputs: &InputState) -> StaticBool {
    match expression.trim() {
        "${{ false }}" => StaticBool::False,
        "${{ true }}" => StaticBool::True,
        expression => resolve_input_expression(strip_expression(expression), inputs),
    }
}

fn strip_expression(expression: &str) -> &str {
    expression
        .strip_prefix("${{")
        .and_then(|body| body.strip_suffix("}}"))
        .map(str::trim)
        .unwrap_or(expression)
}

fn resolve_input_expression(expression: &str, inputs: &InputState) -> StaticBool {
    for (operator, equal) in [("==", true), ("!=", false)] {
        if let Some((left, right)) = expression.split_once(operator) {
            let Some(name) = left.trim().strip_prefix("inputs.") else {
                return StaticBool::Unknown;
            };
            let expected = match right.trim() {
                "true" => true,
                "false" => false,
                _ => return StaticBool::Unknown,
            };
            let value = inputs
                .get(&name.trim().to_lowercase())
                .copied()
                .unwrap_or(StaticBool::Unknown)
                .equals(expected);
            return if equal { value } else { value.negate() };
        }
    }
    if let Some(name) = expression.strip_prefix("inputs.") {
        return inputs
            .get(&name.trim().to_lowercase())
            .copied()
            .unwrap_or(StaticBool::Unknown);
    }
    if let Some(name) = expression
        .strip_prefix('!')
        .map(str::trim)
        .and_then(|operand| operand.strip_prefix("inputs."))
    {
        return inputs
            .get(&name.trim().to_lowercase())
            .copied()
            .unwrap_or(StaticBool::Unknown)
            .negate();
    }
    StaticBool::Unknown
}

impl StaticBool {
    fn negate(self) -> Self {
        match self {
            Self::False => Self::True,
            Self::True => Self::False,
            Self::Unknown => Self::Unknown,
        }
    }

    fn equals(self, expected: bool) -> Self {
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
