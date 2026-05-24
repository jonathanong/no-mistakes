use super::*;
use crate::playwright::analysis::text_types::AppTextKind;
use crate::playwright::analysis::types::SelectorRef;

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
        false,
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
        false,
    ));
    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        true,
        &test_name,
        &describe_path,
        false,
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
        false,
    ));
}

#[test]
fn hook_route_signal_matches_hook_locator_without_test_name() {
    let route_test_name = None;
    let route_describe_path = Arc::new(vec!["suite".to_string()]);
    let test_name = None;
    let describe_path = Arc::new(vec!["suite".to_string()]);

    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        true,
        &test_name,
        &describe_path,
        true,
    ));
}

#[test]
fn adjacent_selector_signal_only_uses_preceding_selectors() {
    let test_file = Arc::new("tests/app.spec.ts".to_string());
    let test_name = Some(Arc::new("uses text locator".to_string()));
    let describe_path = Arc::new(vec!["suite".to_string()]);
    let app_file = Arc::new("web/app/page.tsx".to_string());
    let app_text = AppTextTarget {
        file: "web/app/page.tsx".into(),
        app_file: app_file.clone(),
        kind: AppTextKind::AccessibleName,
        role: Some("button".to_string()),
        text: "Save".to_string(),
        hidden: false,
        selector_refs: vec![SelectorRef {
            attribute: "data-pw".to_string(),
            value: "save-button".to_string(),
        }],
    };
    let selector = |line| Edge::Selector {
        test_file: test_file.clone(),
        test_name: test_name.clone(),
        describe_path: describe_path.clone(),
        app_file: app_file.clone(),
        attribute: "data-pw".to_string(),
        value: "save-button".to_string(),
        selector: "[data-pw=\"save-button\"]".to_string(),
        line,
    };

    assert!(has_adjacent_selector_signal(
        &[selector(10)],
        &test_file,
        &test_name,
        &describe_path,
        15,
        &app_text,
    ));
    assert!(!has_adjacent_selector_signal(
        &[selector(16)],
        &test_file,
        &test_name,
        &describe_path,
        15,
        &app_text,
    ));
}
