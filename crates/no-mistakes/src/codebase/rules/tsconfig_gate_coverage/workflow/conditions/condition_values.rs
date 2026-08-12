use super::{
    event_action_value, event_base_ref_value, event_head_ref_value, event_name_value,
    event_ref_name_value, event_ref_type_value, functions,
    input_value::comparison_literal,
    literals::{job_status_value, status_function_bool},
    logical, pull_request_merged_value,
    resolution::condition_input_value,
    resolution::{
        github_base_ref, github_event_action, github_event_name, github_head_ref,
        github_pull_request_merged, github_ref, github_ref_name, github_ref_type, github_workflow,
        job_status,
    },
    static_json::literal_from_json_static_value,
    workflow_value, ConditionStatus, EnvironmentState, InputState, StaticBool, StaticValue,
};

mod comparison;
mod static_bool;
pub(super) use comparison::comparison_bool;
use static_bool::static_bool_value;
pub(super) fn condition_value(
    operand: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: impl Into<ConditionStatus>,
) -> Option<StaticValue> {
    let status = status.into();
    let operand = operand.trim();
    if let Some(value) = status_function_bool(operand, status) {
        return match value {
            StaticBool::False => Some(StaticValue::Bool(false)),
            StaticBool::True => Some(StaticValue::Bool(true)),
            StaticBool::TruthyNonBoolean | StaticBool::Invalid | StaticBool::Unknown => None,
        };
    }
    if github_event_name(operand) {
        return event_name_value(inputs);
    }
    if github_event_action(operand) {
        return event_action_value(inputs);
    }
    if github_workflow(operand) {
        return workflow_value(inputs);
    }
    if github_pull_request_merged(operand) {
        return pull_request_merged_value(inputs);
    }
    if github_ref(operand) {
        return Some(
            inputs
                .get(super::inputs::REF_KEY)
                .cloned()
                .unwrap_or(StaticValue::Unknown),
        );
    }
    if github_ref_name(operand) {
        return event_ref_name_value(inputs);
    }
    if github_ref_type(operand) {
        return event_ref_type_value(inputs);
    }
    if job_status(operand) {
        return Some(job_status_value(status));
    }
    if github_base_ref(operand) {
        return event_base_ref_value(inputs);
    }
    if github_head_ref(operand) {
        return event_head_ref_value(inputs);
    }
    literal_from_json_static_value(operand)
        .or_else(|| super::static_values::static_from_json_expression(operand, inputs, environment))
        .or_else(|| super::static_values::static_to_json_expression(operand, inputs, environment))
        .or_else(|| condition_input_value(operand, inputs, environment))
        .or_else(|| comparison_literal(operand))
        .or_else(|| {
            logical::outer_parentheses_body(operand)
                .and_then(|operand| condition_value(operand, inputs, environment, status))
        })
        .or_else(|| logical_value(operand, inputs, environment, status))
        .or_else(|| comparison_bool(operand, inputs, environment, status).map(static_bool_value))
        .or_else(|| {
            operand.strip_prefix('!').map(str::trim).map(|operand| {
                static_bool_value(
                    super::expression_bool_with_status_and_environment(
                        operand,
                        inputs,
                        environment,
                        status,
                    )
                    .negate(),
                )
            })
        })
        .or_else(|| functions::static_format_value(operand, inputs, environment, status))
        .or_else(|| functions::static_join_value(operand, inputs, environment, status))
        .or_else(|| {
            functions::static_function_bool(operand, inputs, environment, status)
                .map(static_bool_value)
        })
}
fn logical_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> Option<StaticValue> {
    let (left, right, operator) = logical::logical_operands(expression)?;
    let left = condition_value(left, inputs, environment, status)?;
    match (operator, left.clone().truthiness()) {
        (logical::LogicalOperator::Or, StaticBool::False) => {
            condition_value(right, inputs, environment, status)
        }
        (logical::LogicalOperator::Or, StaticBool::True | StaticBool::TruthyNonBoolean) => {
            Some(left)
        }
        (logical::LogicalOperator::And, StaticBool::False) => Some(left),
        (logical::LogicalOperator::And, StaticBool::True | StaticBool::TruthyNonBoolean) => {
            condition_value(right, inputs, environment, status)
        }
        (_, StaticBool::Invalid | StaticBool::Unknown) => None,
    }
}
