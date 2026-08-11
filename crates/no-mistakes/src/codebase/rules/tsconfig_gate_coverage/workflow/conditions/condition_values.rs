use super::{
    event_action_value, event_base_ref_value, event_name_value, event_ref_name_value, functions,
    input_value::comparison_literal,
    literals::status_function_bool,
    logical,
    resolution::condition_input_value,
    resolution::{
        github_base_ref, github_event_action, github_event_name, github_ref, github_ref_name,
    },
    static_json::literal_from_json_static_value,
    ConditionStatus, EnvironmentState, InputState, StaticBool, StaticValue,
};

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
    if github_base_ref(operand) {
        return event_base_ref_value(inputs);
    }
    literal_from_json_static_value(operand)
        .or_else(|| super::static_values::static_from_json_expression(operand, inputs, environment))
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
        .or_else(|| functions::static_case_value(operand, inputs, environment, status))
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

pub(super) fn comparison_bool(
    expression: &str,
    inputs: &InputState,
    environment: &EnvironmentState,
    status: ConditionStatus,
) -> Option<StaticBool> {
    let (left, right, comparison) = logical::comparison_operands(expression)?;
    if matches!(
        comparison,
        logical::Comparison::Equal | logical::Comparison::NotEqual
    ) && known_not_skipped_comparison(left, right, inputs)
    {
        let equal = StaticBool::False;
        return Some(if matches!(comparison, logical::Comparison::Equal) {
            equal
        } else {
            equal.negate()
        });
    }
    if matches!(
        comparison,
        logical::Comparison::Equal | logical::Comparison::NotEqual
    ) && (github_ref(left) || github_ref(right))
    {
        let other = if github_ref(left) { right } else { left };
        if let Some(StaticValue::String(reference)) = comparison_literal(other) {
            if inputs
                .get(super::inputs::REF_EXCLUSIONS_KEY)
                .is_some_and(|excluded| {
                    matches!(excluded, StaticValue::Sequence(values) if values.contains(&StaticValue::String(reference.clone())))
                })
            {
                let equal = StaticBool::False;
                return Some(if matches!(comparison, logical::Comparison::Equal) {
                    equal
                } else {
                    equal.negate()
                });
            }
            let is_pull_request_merge = inputs
                .get(super::inputs::REF_KIND_KEY)
                .is_some_and(|kind| kind == &StaticValue::String("pull-request-merge".into()));
            if is_pull_request_merge && !reference.starts_with("refs/pull/") {
                let equal = StaticBool::False;
                return Some(if matches!(comparison, logical::Comparison::Equal) {
                    equal
                } else {
                    equal.negate()
                });
            }
        }
    }
    let actual = condition_value(left, inputs, environment, status)?;
    let expected = condition_value(right, inputs, environment, status)?;
    Some(match comparison {
        logical::Comparison::Equal => actual.equals(&expected),
        logical::Comparison::NotEqual => actual.equals(&expected).negate(),
        logical::Comparison::LessThan => actual.less_than(&expected),
        logical::Comparison::LessThanOrEqual => actual.less_than_or_equal(&expected),
        logical::Comparison::GreaterThan => expected.less_than(&actual),
        logical::Comparison::GreaterThanOrEqual => expected.less_than_or_equal(&actual),
    })
}

fn known_not_skipped_comparison(left: &str, right: &str, inputs: &InputState) -> bool {
    [(left, right), (right, left)]
        .into_iter()
        .any(|(actual, expected)| {
            super::resolution::needs_result_is_known_not_skipped(actual, inputs)
                && matches!(
                    comparison_literal(expected),
                    Some(StaticValue::String(value)) if value.eq_ignore_ascii_case("skipped")
                )
        })
}

fn static_bool_value(value: StaticBool) -> StaticValue {
    match value {
        StaticBool::False => StaticValue::Bool(false),
        StaticBool::True => StaticValue::Bool(true),
        StaticBool::Invalid => StaticValue::Invalid,
        StaticBool::TruthyNonBoolean | StaticBool::Unknown => StaticValue::Unknown,
    }
}
