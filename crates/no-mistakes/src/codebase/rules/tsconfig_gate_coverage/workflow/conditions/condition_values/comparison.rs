use super::{
    comparison_literal, condition_value, github_ref, logical, ConditionStatus, EnvironmentState,
    InputState, StaticBool, StaticValue,
};

pub(in super::super) fn comparison_bool(
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
        return Some(compared_false(comparison));
    }
    if matches!(
        comparison,
        logical::Comparison::Equal | logical::Comparison::NotEqual
    ) && (github_ref(left) || github_ref(right))
        && impossible_ref_comparison(left, right, inputs)
    {
        return Some(compared_false(comparison));
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

fn compared_false(comparison: logical::Comparison) -> StaticBool {
    match comparison {
        logical::Comparison::Equal => StaticBool::False,
        logical::Comparison::NotEqual => StaticBool::True,
        _ => unreachable!("only equality comparisons call this helper"),
    }
}

fn impossible_ref_comparison(left: &str, right: &str, inputs: &InputState) -> bool {
    let other = if github_ref(left) { right } else { left };
    let Some(StaticValue::String(reference)) = comparison_literal(other) else {
        return false;
    };
    if inputs.get(super::super::inputs::REF_EXCLUSIONS_KEY).is_some_and(|excluded| {
        matches!(excluded, StaticValue::Sequence(values) if values.contains(&StaticValue::String(reference.clone())))
    }) {
        return true;
    }
    match inputs.get(super::super::inputs::REF_SHAPE_KEY) {
        Some(StaticValue::String(kind)) if kind == "branch" => {
            !reference.starts_with("refs/heads/")
        }
        Some(StaticValue::String(kind)) if kind == "tag" => !reference.starts_with("refs/tags/"),
        Some(StaticValue::String(kind)) if kind == "pull-request-merge" => {
            !reference.starts_with("refs/pull/")
        }
        _ => false,
    }
}

fn known_not_skipped_comparison(left: &str, right: &str, inputs: &InputState) -> bool {
    [(left, right), (right, left)].into_iter().any(|(actual, expected)| {
        super::super::resolution::needs_result_is_known_not_skipped(actual, inputs)
            && matches!(comparison_literal(expected), Some(StaticValue::String(value)) if value.eq_ignore_ascii_case("skipped"))
    })
}
