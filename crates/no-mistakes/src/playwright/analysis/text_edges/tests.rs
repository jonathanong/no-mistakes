use super::*;
use crate::playwright::analysis::context::{RouteIndex, SelectorIndex, TestAnalysisContext};
use crate::playwright::analysis::text_types::AppTextKind;
use crate::playwright::analysis::types::SelectorRef;
use crate::playwright::playwright_tests::TestOccurrenceScope;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn text_target(kind: AppTextKind, text: &str, role: Option<&str>, hidden: bool) -> AppTextTarget {
    AppTextTarget {
        file: "web/app/page.tsx".into(),
        app_file: Arc::new("web/app/page.tsx".to_string()),
        kind,
        role: role.map(str::to_string),
        text: text.to_string(),
        hidden,
        selector_refs: vec![SelectorRef {
            attribute: "data-pw".to_string(),
            value: "save-button".to_string(),
        }],
    }
}

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
        TestOccurrenceScope::Test,
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
        TestOccurrenceScope::Test,
    ));
    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        true,
        &test_name,
        &describe_path,
        TestOccurrenceScope::Test,
    ));
}

#[test]
fn route_signal_matches_unnamed_file_scope_pairs() {
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
        TestOccurrenceScope::File,
    ));
    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        false,
        &test_name,
        &describe_path,
        TestOccurrenceScope::Test,
    ));
}

#[test]
fn route_signal_matches_unnamed_describe_scope_pairs() {
    let route_test_name = None;
    let route_describe_path = Arc::new(vec!["suite".to_string()]);
    let test_name = None;
    let describe_path = Arc::new(vec!["suite".to_string()]);

    assert!(route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        false,
        &test_name,
        &describe_path,
        TestOccurrenceScope::Test,
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
        TestOccurrenceScope::Hook,
    ));
}

#[test]
fn hook_route_signal_matches_unnamed_test_callbacks() {
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
        TestOccurrenceScope::Test,
    ));
    assert!(!route_signal_matches_test(
        &route_test_name,
        &route_describe_path,
        true,
        &test_name,
        &describe_path,
        TestOccurrenceScope::File,
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

#[test]
fn adjacent_selector_signal_rejects_file_scope_pairs() {
    let test_file = Arc::new("tests/app.spec.ts".to_string());
    let test_name = None;
    let describe_path = Arc::new(vec![]);
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
    let selector = Edge::Selector {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: describe_path.clone(),
        app_file,
        attribute: "data-pw".to_string(),
        value: "save-button".to_string(),
        selector: "[data-pw=\"save-button\"]".to_string(),
        line: 10,
    };

    assert!(!has_adjacent_selector_signal(
        &[selector],
        &test_file,
        &test_name,
        &describe_path,
        15,
        &app_text,
    ));
}

#[test]
fn hook_route_signal_ignores_declaration_line_order() {
    let test_file = Arc::new("tests/app.spec.ts".to_string());
    let route_file = Arc::new("web/app/page.tsx".to_string());
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
    let mut reachable = BTreeMap::new();
    reachable.insert(route_file.clone(), BTreeSet::from([app_file.clone()]));
    let route_index = RouteIndex::default();
    let selector_index = SelectorIndex::default();
    let selector_regexes =
        crate::playwright::selectors::compile_selector_regexes(&[], &Default::default());
    let context = TestAnalysisContext {
        root: Path::new("/repo"),
        route_index: &route_index,
        app_selector_targets: &[],
        selector_index: &selector_index,
        app_text_targets: &[],
        route_reachable_files: &reachable,
        navigation_helpers: &[],
        selector_regexes: &selector_regexes,
        test_policy: crate::playwright::playwright_tests::TestPolicy::default(),
    };
    let locator_test_name = Some(Arc::new("uses text locator".to_string()));
    let locator_describe_path = Arc::new(vec!["suite".to_string()]);
    let scope = LocatorTestScope {
        test_name: &locator_test_name,
        describe_path: &locator_describe_path,
        scope: TestOccurrenceScope::Test,
    };
    let hook_route = |line| Edge::Route {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: Arc::new(vec!["suite".to_string()]),
        route_file: route_file.clone(),
        route: Arc::new("/".to_string()),
        url: Arc::new("/".to_string()),
        hook: true,
        line,
    };

    assert!(has_reachable_route_signal(
        &[hook_route(10)],
        &test_file,
        &scope,
        15,
        &app_text,
        &context,
    ));
    assert!(has_reachable_route_signal(
        &[hook_route(16)],
        &test_file,
        &scope,
        15,
        &app_text,
        &context,
    ));
}

#[test]
fn teardown_text_locators_do_not_create_text_edges() {
    let test_file = Arc::new("tests/app.spec.ts".to_string());
    let app_file = Arc::new("web/app/page.tsx".to_string());
    let app_text = AppTextTarget {
        file: "web/app/page.tsx".into(),
        app_file: app_file.clone(),
        kind: AppTextKind::VisibleText,
        role: None,
        text: "Cleanup text".to_string(),
        hidden: false,
        selector_refs: vec![SelectorRef {
            attribute: "data-pw".to_string(),
            value: "cleanup".to_string(),
        }],
    };
    let route_index = RouteIndex::default();
    let selector_index = SelectorIndex::default();
    let reachable = BTreeMap::new();
    let selector_regexes =
        crate::playwright::selectors::compile_selector_regexes(&[], &Default::default());
    let context = TestAnalysisContext {
        root: Path::new("/repo"),
        route_index: &route_index,
        app_selector_targets: &[],
        selector_index: &selector_index,
        app_text_targets: std::slice::from_ref(&app_text),
        route_reachable_files: &reachable,
        navigation_helpers: &[],
        selector_regexes: &selector_regexes,
        test_policy: crate::playwright::playwright_tests::TestPolicy::default(),
    };
    let mut edges = vec![Edge::Selector {
        test_file: test_file.clone(),
        test_name: None,
        describe_path: Arc::new(vec![]),
        app_file,
        attribute: "data-pw".to_string(),
        value: "cleanup".to_string(),
        selector: "[data-pw=\"cleanup\"]".to_string(),
        line: 1,
    }];
    let locator = crate::playwright::analysis::text_types::PlaywrightTextLocator {
        kind: crate::playwright::analysis::text_types::LocatorKind::Text,
        role: None,
        text: "Cleanup text".to_string(),
        locator: "getByText(Cleanup text)".to_string(),
        exact: true,
        include_hidden: false,
    };

    append_locator_text_edges(
        &mut edges,
        &test_file,
        &context,
        vec![crate::playwright::playwright_tests::TestOccurrence {
            value: locator,
            status: crate::playwright::playwright_tests::TestStatus::Active,
            scope: TestOccurrenceScope::TeardownHook,
            test_name: None,
            describe_path: Vec::new(),
            line: 2,
        }],
    );

    assert!(!edges
        .iter()
        .any(|edge| matches!(edge, Edge::LocatorText { .. })));
}

#[test]
fn app_text_index_filters_exact_and_fuzzy_candidates_by_kind_role() {
    let targets = vec![
        text_target(AppTextKind::AccessibleName, "Save", Some("button"), false),
        text_target(AppTextKind::AccessibleName, "Save", Some("link"), false),
        text_target(AppTextKind::VisibleText, "Save now", None, false),
        text_target(AppTextKind::Label, "Email", Some("textbox"), false),
    ];
    let index = AppTextIndex::new(&targets);
    let exact = PlaywrightTextLocator {
        kind: crate::playwright::analysis::text_types::LocatorKind::Role,
        role: Some("button".to_string()),
        text: "Save".to_string(),
        locator: "getByRole(button, name: Save)".to_string(),
        exact: true,
        include_hidden: false,
    };
    let fuzzy = PlaywrightTextLocator {
        kind: crate::playwright::analysis::text_types::LocatorKind::Text,
        role: None,
        text: "save".to_string(),
        locator: "getByText(save)".to_string(),
        exact: false,
        include_hidden: false,
    };
    let label = PlaywrightTextLocator {
        kind: crate::playwright::analysis::text_types::LocatorKind::Label,
        role: None,
        text: "Email".to_string(),
        locator: "getByLabel(Email)".to_string(),
        exact: true,
        include_hidden: false,
    };
    let role_without_name = PlaywrightTextLocator {
        kind: crate::playwright::analysis::text_types::LocatorKind::Role,
        role: None,
        text: "Save".to_string(),
        locator: "getByRole(roleExpression, name: Save)".to_string(),
        exact: true,
        include_hidden: false,
    };

    assert_eq!(index.candidates(&exact).len(), 1);
    assert_eq!(index.candidates(&fuzzy).len(), 1);
    assert_eq!(index.candidates(&fuzzy)[0].text, "Save now");
    assert_eq!(index.candidates(&label).len(), 1);
    assert_eq!(index.candidates(&label)[0].text, "Email");
    assert!(index.candidates(&role_without_name).is_empty());
}
