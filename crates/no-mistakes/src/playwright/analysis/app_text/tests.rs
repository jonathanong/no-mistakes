use super::*;
use crate::playwright::analysis::text_types::normalize_locator_text;
use std::collections::BTreeMap;

fn settings() -> Settings {
    Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        navigation_helpers: vec![],
        selector_attributes: vec!["data-pw".to_string()],
        component_selector_attributes: BTreeMap::from([(
            "testId".to_string(),
            "data-pw".to_string(),
        )]),
        html_ids: false,
        selector_roots: vec![],
        selector_include: vec![],
        selector_exclude: vec![],
    }
}

#[test]
fn extracts_app_text_targets_from_fixture_jsx_shapes() {
    let path = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "app-text-targets.tsx",
    ]);
    let root = path.parent().unwrap();
    let source = std::fs::read_to_string(&path).expect("fixture should read");
    let targets =
        extract_app_text_targets(root, &path, &source, &settings()).expect("fixture parses");

    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Label
            && target.text == "Email address"
            && target.selector_refs[0].value == "email-label"
    }));
    assert!(targets
        .iter()
        .any(|target| target.kind == AppTextKind::AccessibleName && target.text == "Search field"));
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Placeholder
            && target.text == "Search"
            && target.selector_refs[0].value == "search-input"
    }));
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::AccessibleName && target.text == "Company logo"
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Save"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "save-button")
    }));
    assert!(targets.iter().any(|target| target.text == "String child"));
}

#[test]
fn normalize_locator_text_rejects_blank_text() {
    assert_eq!(normalize_locator_text(" \n\t "), None);
}
