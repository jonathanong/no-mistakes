fn route_helper_ref_wrapped_patterns(
    helper_ref: &crate::codebase::ts_routes::refs::RouteHelperRef,
    helper_patterns: Vec<String>,
) -> Vec<String> {
    let Some(wrapper) = &helper_ref.wrapper_pattern else {
        return helper_patterns;
    };
    helper_patterns
        .into_iter()
        .filter_map(|helper_pattern| route_helper_ref_wrapped_pattern(wrapper, &helper_pattern))
        .collect()
}

fn route_helper_ref_wrapped_pattern(wrapper: &str, helper_pattern: &str) -> Option<String> {
    use crate::codebase::ts_routes::refs::{
        normalize_next_pathname_pattern, should_skip, ROUTE_HELPER_REF_PATTERN_MARKER,
    };

    let (prefix, suffix) = wrapper.split_once(ROUTE_HELPER_REF_PATTERN_MARKER)?;
    let mut pattern = prefix.to_string();
    append_route_helper_ref_pattern_part(&mut pattern, helper_pattern);
    append_route_helper_ref_pattern_part(&mut pattern, suffix);
    if !pattern.starts_with('/') || should_skip(&pattern) {
        return None;
    }
    Some(normalize_next_pathname_pattern(&pattern))
}

fn append_route_helper_ref_pattern_part(pattern: &mut String, part: &str) {
    if pattern.ends_with('/') && part.starts_with('/') {
        pattern.push_str(part.trim_start_matches('/'));
    } else {
        pattern.push_str(part);
    }
}
