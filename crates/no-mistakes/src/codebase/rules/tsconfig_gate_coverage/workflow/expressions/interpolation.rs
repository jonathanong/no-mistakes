pub(in super::super) fn interpolated_expression_valid(value: &str) -> bool {
    super::interpolated_expression_valid_for_contexts(value, None, false)
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
        let end = super::interpolated_expression_end(body)?;
        normalized.push_str(marker);
        remaining = &body[end + "}}".len()..];
    }
    normalized.push_str(remaining);
    Some(normalized)
}
