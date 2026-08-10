use super::values::only_keys;
use serde_yaml::{Mapping, Value};

use super::super::super::super::expressions::{
    condition_expression_valid, interpolated_expression_valid,
};

pub(super) fn strategy_shape_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.as_mapping().is_some_and(|strategy| {
            !strategy.is_empty()
                && only_keys(strategy, &["fail-fast", "max-parallel", "matrix"])
                && strategy.get("fail-fast").is_none_or(|value| {
                    value.is_bool()
                        || value
                            .as_str()
                            .is_some_and(super::super::super::super::complete_expression)
                })
                && strategy.get("max-parallel").is_none_or(|value| {
                    value.as_u64().is_some_and(|value| value > 0)
                        || value
                            .as_str()
                            .is_some_and(super::super::super::super::complete_expression)
                })
        })
    })
}

pub(super) fn string_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping
        .get(field)
        .is_none_or(|value| value.as_str().is_some_and(interpolated_expression_valid))
}

pub(crate) fn condition_field_valid(value: Option<&Value>) -> bool {
    value.is_none_or(|value| {
        value.is_bool() || value.as_str().is_some_and(condition_expression_valid)
    })
}

pub(super) fn bool_or_expression_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(|value| {
        value.is_bool()
            || value
                .as_str()
                .is_some_and(super::super::super::super::complete_expression)
    })
}

pub(super) fn number_or_expression_field_valid(mapping: &Mapping, field: &str) -> bool {
    mapping.get(field).is_none_or(|value| {
        value
            .as_u64()
            .is_some_and(|minutes| (1..=360).contains(&minutes))
            || value
                .as_str()
                .is_some_and(super::super::super::super::complete_expression)
    })
}

pub(super) fn scalar_value_valid(value: &Value) -> bool {
    matches!(value, Value::Bool(_) | Value::Number(_))
        || value.as_str().is_some_and(interpolated_expression_valid)
}
