use super::*;

#[test]
fn route_signal_matches_exact_test_scope() {
    let route_test_name = Some(Arc::new("visits home".to_string()));
    let route_describe_path = Arc::new(vec!["suite".to_string()]);
    let test_name = Some(Arc::new("visits home".to_string()));
    let describe_path = Arc::new(vec!["suite".to_string()]);

    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        false,
        &test_name,
        &describe_path,
    ));
}

#[test]
fn route_signal_fallback_requires_hook_scope() {
    let route_test_name = None;
    let route_describe_path = Arc::new(vec!["suite".to_string()]);
    let test_name = Some(Arc::new("visits home".to_string()));
    let describe_path = Arc::new(vec!["suite".to_string()]);

    assert!(!route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        false,
        &test_name,
        &describe_path,
    ));
    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        true,
        &test_name,
        &describe_path,
    ));
}

#[test]
fn route_signal_does_not_match_unnamed_file_scope_pairs() {
    let route_test_name = None;
    let route_describe_path = Arc::new(vec![]);
    let test_name = None;
    let describe_path = Arc::new(vec![]);

    assert!(!route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        false,
        &test_name,
        &describe_path,
    ));
}
