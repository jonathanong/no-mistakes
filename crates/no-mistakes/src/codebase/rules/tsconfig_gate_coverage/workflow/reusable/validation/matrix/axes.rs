use serde_yaml::Value;

pub(super) enum StaticMatrixAxes {
    Static(Vec<(String, Vec<Value>)>),
    Dynamic,
    Invalid,
}

pub(super) fn static_matrix_axes(mapping: &serde_yaml::Mapping) -> StaticMatrixAxes {
    let mut axes = Vec::new();
    let mut dynamic = false;
    for (name, values) in mapping {
        if matches!(name.as_str(), Some("include" | "exclude")) {
            continue;
        }
        let Some(name) = name.as_str() else {
            return StaticMatrixAxes::Invalid;
        };
        match values {
            Value::Sequence(values) => match resolved_static_axis_values(values) {
                ResolvedStaticAxisValues::Static(values) if values.is_empty() => {
                    return StaticMatrixAxes::Invalid;
                }
                ResolvedStaticAxisValues::Static(values) => axes.push((name.to_string(), values)),
                ResolvedStaticAxisValues::Dynamic => dynamic = true,
                ResolvedStaticAxisValues::Invalid => return StaticMatrixAxes::Invalid,
            },
            Value::String(expression) if super::matrix_expression_valid(expression) => {
                dynamic = true;
            }
            _ => return StaticMatrixAxes::Invalid,
        }
    }
    if dynamic {
        StaticMatrixAxes::Dynamic
    } else if axes.len() > super::STATIC_MATRIX_AXIS_LIMIT {
        StaticMatrixAxes::Invalid
    } else {
        StaticMatrixAxes::Static(axes)
    }
}

/// Matrix axis entries are evaluated individually. Resolve context-free
/// expressions so the cartesian product and include/exclude matching use the
/// same typed values GitHub Actions sees at runtime.
enum ResolvedStaticAxisValues {
    Static(Vec<Value>),
    Dynamic,
    Invalid,
}

fn resolved_static_axis_values(values: &[Value]) -> ResolvedStaticAxisValues {
    let mut resolved = Vec::with_capacity(values.len());
    let mut dynamic = false;
    for value in values {
        match value {
            Value::Sequence(_) => dynamic = true,
            Value::String(expression)
                if super::super::super::super::complete_expression(expression) =>
            {
                if !super::matrix_expression_valid(expression) {
                    return ResolvedStaticAxisValues::Invalid;
                }
                let Some(value) =
                    super::super::super::super::complete_literal_expression_value(expression)
                else {
                    dynamic = true;
                    continue;
                };
                if matches!(value, Value::Sequence(_)) {
                    dynamic = true;
                    continue;
                }
                resolved.push(value);
            }
            Value::String(expression)
                if expression.contains("${{") || expression.contains("}}") =>
            {
                if !super::matrix_interpolated_expression_valid(expression) {
                    return ResolvedStaticAxisValues::Invalid;
                }
                dynamic = true;
            }
            value => resolved.push(value.clone()),
        }
    }
    if dynamic {
        ResolvedStaticAxisValues::Dynamic
    } else {
        ResolvedStaticAxisValues::Static(resolved)
    }
}
