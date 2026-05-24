use std::sync::Arc;

pub(super) fn route_signal_matches_test(
    route_test_name: &Option<Arc<String>>,
    route_describe_path: &Arc<Vec<String>>,
    route_is_hook: bool,
    test_name: &Option<Arc<String>>,
    describe_path: &Arc<Vec<String>>,
    locator_is_hook: bool,
) -> bool {
    let exact_test_scope = route_test_name.is_some()
        && route_test_name == test_name
        && route_describe_path == describe_path;
    let unnamed_test_scope = route_test_name.is_none()
        && test_name.is_none()
        && !route_is_hook
        && !locator_is_hook
        && !describe_path.is_empty()
        && route_describe_path == describe_path;
    let hook_scope = route_test_name.is_none()
        && route_is_hook
        && (test_name.is_some() || locator_is_hook)
        && describe_path_starts_with(describe_path, route_describe_path);
    exact_test_scope || unnamed_test_scope || hook_scope
}

fn describe_path_starts_with(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
}
