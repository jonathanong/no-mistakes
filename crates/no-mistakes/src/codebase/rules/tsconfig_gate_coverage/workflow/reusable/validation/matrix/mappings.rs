use serde_yaml::Value;

pub(super) enum StaticMappings {
    Static(Vec<serde_yaml::Mapping>),
    Dynamic,
    Invalid,
}

enum ResolvedStaticMapping {
    Static(serde_yaml::Mapping),
    Dynamic,
    Invalid,
}

pub(super) fn static_mappings(value: Option<&Value>) -> StaticMappings {
    match value {
        Some(Value::Sequence(items)) if !items.is_empty() => {
            let mut mappings = Vec::with_capacity(items.len());
            let mut dynamic = false;
            for item in items {
                let Some(mapping) = item.as_mapping() else {
                    return StaticMappings::Invalid;
                };
                match resolved_static_mapping(mapping) {
                    ResolvedStaticMapping::Static(mapping) => mappings.push(mapping),
                    ResolvedStaticMapping::Dynamic => dynamic = true,
                    ResolvedStaticMapping::Invalid => return StaticMappings::Invalid,
                }
            }
            if dynamic {
                StaticMappings::Dynamic
            } else {
                StaticMappings::Static(mappings)
            }
        }
        Some(Value::Sequence(_)) => StaticMappings::Invalid,
        Some(Value::String(expression)) if super::matrix_expression_valid(expression) => {
            if super::super::super::super::complete_expression_may_be_mapping(expression) {
                StaticMappings::Dynamic
            } else {
                StaticMappings::Invalid
            }
        }
        Some(_) => StaticMappings::Invalid,
        None => StaticMappings::Static(Vec::new()),
    }
}

/// GitHub evaluates a complete literal expression in `include` or `exclude`
/// before it compares or applies the mapping. Unknown expressions deliberately
/// stop static enumeration so a partial expansion cannot credit a skipped job.
fn resolved_static_mapping(mapping: &serde_yaml::Mapping) -> ResolvedStaticMapping {
    let mut resolved = serde_yaml::Mapping::new();
    let mut dynamic = false;
    for (name, value) in mapping {
        match resolved_static_value(value) {
            ResolvedStaticValue::Static(value) => {
                resolved.insert(name.clone(), value);
            }
            ResolvedStaticValue::Dynamic => dynamic = true,
            ResolvedStaticValue::Invalid => return ResolvedStaticMapping::Invalid,
        }
    }
    if dynamic {
        ResolvedStaticMapping::Dynamic
    } else {
        ResolvedStaticMapping::Static(resolved)
    }
}

enum ResolvedStaticValue {
    Static(Value),
    Dynamic,
    Invalid,
}

fn resolved_static_value(value: &Value) -> ResolvedStaticValue {
    let Value::String(expression) = value else {
        return ResolvedStaticValue::Static(value.clone());
    };
    if !super::super::super::super::complete_expression(expression) {
        if expression.contains("${{") || expression.contains("}}") {
            return if super::matrix_interpolated_expression_valid(expression) {
                ResolvedStaticValue::Dynamic
            } else {
                ResolvedStaticValue::Invalid
            };
        }
        return ResolvedStaticValue::Static(value.clone());
    }
    if !super::matrix_expression_valid(expression) {
        return ResolvedStaticValue::Invalid;
    }
    super::super::super::super::complete_literal_expression_value(expression)
        .map_or(ResolvedStaticValue::Dynamic, ResolvedStaticValue::Static)
}
