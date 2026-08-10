use crate::codebase::workflow_topology::model::{
    JsonScalar, WorkflowCallContract, WorkflowCallInputType,
};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum StaticBool {
    False,
    True,
    Unknown,
}

pub(super) type InputState = BTreeMap<String, StaticBool>;

pub(super) fn direct_inputs(contract: Option<&WorkflowCallContract>) -> InputState {
    contract
        .into_iter()
        .flat_map(|contract| &contract.inputs)
        .filter(|(_, declaration)| declaration.input_type == Some(WorkflowCallInputType::Boolean))
        .map(|(name, _)| (name.clone(), StaticBool::False))
        .collect()
}

pub(super) fn callee_inputs(
    contract: Option<&WorkflowCallContract>,
    call_job: &Value,
    parent: &InputState,
) -> Option<InputState> {
    inputs_from_contract(contract?, call_job.get("with"), parent)
}

fn inputs_from_contract(
    contract: &WorkflowCallContract,
    bindings: Option<&Value>,
    parent: &InputState,
) -> Option<InputState> {
    let binding_map = match bindings {
        Some(Value::Mapping(mapping)) => Some(mapping),
        Some(_) => return None,
        None => None,
    };
    if binding_map.is_some_and(|mapping| {
        mapping
            .keys()
            .filter_map(Value::as_str)
            .any(|name| !contract.inputs.contains_key(name))
            || mapping.keys().any(|key| key.as_str().is_none())
    }) {
        return None;
    }
    for (name, declaration) in &contract.inputs {
        let input_type = declaration.input_type?;
        let binding = binding_map.and_then(|mapping| mapping.get(Value::String(name.clone())));
        if binding.is_none() && declaration.required && declaration.default.is_none() {
            return None;
        }
        if declaration
            .default
            .as_ref()
            .is_some_and(|default| !default_matches_type(default, input_type))
        {
            return None;
        }
        if let Some(binding) = binding {
            if !binding_matches_type(binding, input_type) {
                return None;
            }
        }
    }
    Some(
        contract
            .inputs
            .iter()
            .filter(|(_, declaration)| {
                declaration.input_type == Some(WorkflowCallInputType::Boolean)
            })
            .map(|(name, declaration)| {
                let binding =
                    binding_map.and_then(|mapping| mapping.get(Value::String(name.clone())));
                let state = binding
                    .map(|value| binding_bool(value, parent))
                    .unwrap_or_else(|| match declaration.default.as_ref() {
                        Some(JsonScalar::Bool(value)) => StaticBool::from(*value),
                        Some(_) => StaticBool::Unknown,
                        None => StaticBool::False,
                    });
                (name.clone(), state)
            })
            .collect(),
    )
}

fn default_matches_type(value: &JsonScalar, input_type: WorkflowCallInputType) -> bool {
    matches!(
        (input_type, value),
        (WorkflowCallInputType::Boolean, JsonScalar::Bool(_))
            | (WorkflowCallInputType::Number, JsonScalar::Number(_))
            | (WorkflowCallInputType::String, JsonScalar::Text(_))
    )
}

fn binding_matches_type(value: &Value, input_type: WorkflowCallInputType) -> bool {
    if value
        .as_str()
        .is_some_and(|text| is_complete_expression(text.trim()))
    {
        return true;
    }
    matches!(
        (input_type, value),
        (WorkflowCallInputType::Boolean, Value::Bool(_))
            | (WorkflowCallInputType::Number, Value::Number(_))
            | (WorkflowCallInputType::String, Value::String(_))
    )
}

fn is_complete_expression(value: &str) -> bool {
    value.starts_with("${{") && value.ends_with("}}")
}

fn binding_bool(value: &Value, parent: &InputState) -> StaticBool {
    match value {
        Value::Bool(value) => StaticBool::from(*value),
        Value::String(expression) if is_complete_expression(expression.trim()) => {
            expression_bool(expression, parent)
        }
        _ => StaticBool::Unknown,
    }
}

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
                expression.trim(),
                "always()" | "${{ always() }}" | "!cancelled()" | "${{ !cancelled() }}"
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
                .get(name.trim())
                .copied()
                .unwrap_or(StaticBool::Unknown)
                .equals(expected);
            return if equal { value } else { value.negate() };
        }
    }
    if let Some(name) = expression.strip_prefix("inputs.") {
        return inputs
            .get(name.trim())
            .copied()
            .unwrap_or(StaticBool::Unknown);
    }
    if let Some(name) = expression
        .strip_prefix('!')
        .map(str::trim)
        .and_then(|operand| operand.strip_prefix("inputs."))
    {
        return inputs
            .get(name.trim())
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
