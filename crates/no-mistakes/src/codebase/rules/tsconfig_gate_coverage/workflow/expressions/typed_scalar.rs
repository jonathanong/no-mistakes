use super::{
    complete_expression_contexts_available, complete_expression_type,
    interpolated_expression_contexts_available, literal_value, StaticExpressionType,
};

pub(in super::super) fn typed_scalar_expression_contexts_available(
    value: &str,
    allowed: &[&str],
    expected: StaticExpressionType,
) -> bool {
    if let Some(static_value) = literal_value::complete_literal_expression_value(value) {
        let actual = match static_value {
            serde_yaml::Value::Bool(_) => StaticExpressionType::Boolean,
            serde_yaml::Value::Number(_) => StaticExpressionType::Number,
            serde_yaml::Value::String(_) => StaticExpressionType::String,
            serde_yaml::Value::Null
            | serde_yaml::Value::Sequence(_)
            | serde_yaml::Value::Mapping(_)
            | serde_yaml::Value::Tagged(_) => return false,
        };
        return actual == expected && complete_expression_contexts_available(value, allowed);
    }
    if literal_value::invalid_literal_from_json(value) {
        return false;
    }
    if let Some(actual) = complete_expression_type(value) {
        return complete_expression_contexts_available(value, allowed)
            && (matches!(actual, StaticExpressionType::Dynamic) || actual == expected);
    }
    expected == StaticExpressionType::String
        && interpolated_expression_contexts_available(value, allowed)
}
