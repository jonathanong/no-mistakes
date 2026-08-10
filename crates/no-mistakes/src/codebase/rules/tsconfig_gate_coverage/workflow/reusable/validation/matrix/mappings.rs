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
            for item in items {
                let Some(mapping) = item.as_mapping() else {
                    return StaticMappings::Invalid;
                };
                match resolved_static_mapping(mapping) {
                    ResolvedStaticMapping::Static(mapping) => mappings.push(mapping),
                    ResolvedStaticMapping::Dynamic => return StaticMappings::Dynamic,
                    ResolvedStaticMapping::Invalid => return StaticMappings::Invalid,
                }
            }
            StaticMappings::Static(mappings)
        }
        Some(Value::Sequence(_)) => StaticMappings::Invalid,
        Some(Value::String(expression))
            if super::super::super::super::complete_expression(expression) =>
        {
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
    for (name, value) in mapping {
        match resolved_static_value(value) {
            ResolvedStaticValue::Static(value) => {
                resolved.insert(name.clone(), value);
            }
            ResolvedStaticValue::Dynamic => return ResolvedStaticMapping::Dynamic,
            ResolvedStaticValue::Invalid => return ResolvedStaticMapping::Invalid,
        }
    }
    ResolvedStaticMapping::Static(resolved)
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
        return if expression.trim().starts_with("${{") || expression.trim().ends_with("}}") {
            ResolvedStaticValue::Invalid
        } else {
            ResolvedStaticValue::Static(value.clone())
        };
    }
    super::super::super::super::complete_literal_expression_value(expression)
        .map_or(ResolvedStaticValue::Dynamic, ResolvedStaticValue::Static)
}
