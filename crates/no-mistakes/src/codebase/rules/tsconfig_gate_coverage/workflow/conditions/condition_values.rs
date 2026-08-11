use super::{
    event_action_value, event_name_value, functions,
    input_value::comparison_literal,
    literals::status_function_bool,
    logical,
    resolution::{condition_input_value, literal_from_json_static_value},
    resolution::{github_event_action, github_event_name},
    EnvironmentState, InputState, StaticBool, StaticValue,
};

pub(super) fn condition_value(
    operand: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    success: StaticBool,
) -> Option<StaticValue> {
    let operand = operand.trim();
    if let Some(value) = status_function_bool(operand, success) {
        return match value {
            StaticBool::False => Some(StaticValue::Bool(false)),
            StaticBool::True => Some(StaticValue::Bool(true)),
            StaticBool::TruthyNonBoolean | StaticBool::Unknown => None,
        };
    }
    if github_event_name(operand) {
        return event_name_value(inputs);
    }
    if github_event_action(operand) {
        return event_action_value(inputs);
    }
    literal_from_json_static_value(operand)
        .or_else(|| condition_input_value(operand, inputs, environment))
        .or_else(|| comparison_literal(operand))
        .or_else(|| {
            logical::outer_parentheses_body(operand)
                .and_then(|operand| condition_value(operand, inputs, environment, success))
        })
        .or_else(|| logical_value(operand, inputs, environment, success))
        .or_else(|| comparison_bool(operand, inputs, environment, success).map(static_bool_value))
        .or_else(|| {
            operand.strip_prefix('!').map(str::trim).map(|operand| {
                static_bool_value(
                    super::expression_bool_with_status_and_environment(
                        operand,
                        inputs,
                        environment,
                        success,
                    )
                    .negate(),
                )
            })
        })
        .or_else(|| functions::static_case_value(operand, inputs, environment, success))
        .or_else(|| {
            functions::static_function_bool(operand, inputs, environment, success)
                .map(static_bool_value)
        })
}

fn logical_value(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    success: StaticBool,
) -> Option<StaticValue> {
    let (left, right, operator) = logical::logical_operands(expression)?;
    let left = condition_value(left, inputs, environment, success)?;
    match (operator, left.clone().truthiness()) {
        (logical::LogicalOperator::Or, StaticBool::False) => {
            condition_value(right, inputs, environment, success)
        }
        (logical::LogicalOperator::Or, StaticBool::True | StaticBool::TruthyNonBoolean) => {
            Some(left)
        }
        (logical::LogicalOperator::And, StaticBool::False) => Some(left),
        (logical::LogicalOperator::And, StaticBool::True | StaticBool::TruthyNonBoolean) => {
            condition_value(right, inputs, environment, success)
        }
        (_, StaticBool::Unknown) => None,
    }
}

pub(super) fn comparison_bool(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    success: StaticBool,
) -> Option<StaticBool> {
    let (left, right, comparison) = logical::comparison_operands(expression)?;
    let actual = condition_value(left, inputs, environment, success)?;
    let expected = condition_value(right, inputs, environment, success)?;
    Some(match comparison {
        logical::Comparison::Equal => actual.equals(&expected),
        logical::Comparison::NotEqual => actual.equals(&expected).negate(),
        logical::Comparison::LessThan => actual.less_than(&expected),
        logical::Comparison::LessThanOrEqual => actual.less_than_or_equal(&expected),
        logical::Comparison::GreaterThan => expected.less_than(&actual),
        logical::Comparison::GreaterThanOrEqual => expected.less_than_or_equal(&actual),
    })
}

fn static_bool_value(value: StaticBool) -> StaticValue {
    match value {
        StaticBool::False => StaticValue::Bool(false),
        StaticBool::True => StaticValue::Bool(true),
        StaticBool::TruthyNonBoolean | StaticBool::Unknown => StaticValue::Unknown,
    }
}
