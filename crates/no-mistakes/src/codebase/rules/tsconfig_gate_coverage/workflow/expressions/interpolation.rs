pub(in super::super) fn interpolated_expression_valid(value: &str) -> bool {
    super::validation::interpolated_expression_valid_for_contexts(value, None, false)
}

pub(in super::super) fn opaque_interpolated_expression_form(
    value: &str,
    marker: &str,
) -> Option<String> {
    if marker.is_empty() || value.contains(marker) || !interpolated_expression_valid(value) {
        return None;
    }
    let mut normalized = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find("${{") {
        normalized.push_str(&remaining[..start]);
        let body = &remaining[start + "${{".len()..];
        let end = super::validation::interpolated_expression_end(body)?;
        normalized.push_str(marker);
        remaining = &body[end + "}}".len()..];
    }
    normalized.push_str(remaining);
    Some(normalized)
}

pub(in super::super) enum ContextFreeInterpolation {
    Static(String),
    Dynamic,
    Invalid,
}

/// Reduces each complete, context-free interpolation to its string value.
/// Callers must validate expression syntax and contexts before using this.
pub(in super::super) fn reduce_context_free_interpolations(
    value: &str,
) -> ContextFreeInterpolation {
    let mut normalized = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find("${{") {
        normalized.push_str(&remaining[..start]);
        let body = &remaining[start + "${{".len()..];
        let Some(end) = super::validation::interpolated_expression_end(body) else {
            return ContextFreeInterpolation::Invalid;
        };
        let expression = ["${{ ", body[..end].trim(), " }}"].concat();
        match super::literal_value::complete_literal_expression_value(&expression) {
            Some(serde_yaml::Value::String(value)) => normalized.push_str(&value),
            Some(_) => return ContextFreeInterpolation::Invalid,
            None => return ContextFreeInterpolation::Dynamic,
        }
        remaining = &body[end + "}}".len()..];
    }
    normalized.push_str(remaining);
    ContextFreeInterpolation::Static(normalized)
}

/// Resolves every interpolation using the caller's already-prepared static
/// state. An unavailable expression leaves the whole value unresolved.
pub(in super::super) fn resolve_interpolations(
    value: &str,
    mut resolve: impl FnMut(&str) -> Option<String>,
) -> Option<String> {
    let mut normalized = String::with_capacity(value.len());
    let mut remaining = value;
    while let Some(start) = remaining.find("${{") {
        normalized.push_str(&remaining[..start]);
        let body = &remaining[start + "${{".len()..];
        let end = super::validation::interpolated_expression_end(body)?;
        normalized.push_str(&resolve(body[..end].trim())?);
        remaining = &body[end + "}}".len()..];
    }
    normalized.push_str(remaining);
    Some(normalized)
}
