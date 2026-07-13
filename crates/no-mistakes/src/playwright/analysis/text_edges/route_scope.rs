use crate::playwright::analysis::types::Edge;
use crate::playwright::playwright_tests::TestOccurrenceScope;

fn route_scope_matches(
    route_test_name: Option<&str>,
    route_describe_path: &[String],
    route_is_hook: bool,
    test_name: Option<&str>,
    describe_path: &[String],
    locator_scope: TestOccurrenceScope,
) -> bool {
    let exact_test_scope = route_test_name.is_some()
        && route_test_name == test_name
        && route_describe_path == describe_path;
    let unnamed_test_scope = route_test_name.is_none()
        && test_name.is_none()
        && !route_is_hook
        && locator_scope == TestOccurrenceScope::Test
        && route_describe_path == describe_path;
    let hook_scope = route_test_name.is_none()
        && route_is_hook
        && locator_scope.is_runnable()
        && describe_path_starts_with(describe_path, route_describe_path);
    exact_test_scope || unnamed_test_scope || hook_scope
}

pub(crate) fn route_signal_matches_locator(
    edge: &Edge,
    test_file: &str,
    test_name: Option<&str>,
    describe_path: &[String],
    locator_scope: TestOccurrenceScope,
    locator_line: u32,
) -> bool {
    let Edge::Route {
        test_file: route_test_file,
        test_name: route_test_name,
        describe_path: route_describe_path,
        hook,
        line: route_line,
        ..
    } = edge
    else {
        return false;
    };
    route_test_file.as_str() == test_file
        && route_scope_matches(
            route_test_name.as_deref().map(String::as_str),
            route_describe_path,
            *hook,
            test_name,
            describe_path,
            locator_scope,
        )
        && (*hook || *route_line <= locator_line)
}

fn describe_path_starts_with(path: &[String], prefix: &[String]) -> bool {
    path.len() >= prefix.len() && path.iter().zip(prefix).all(|(left, right)| left == right)
}
