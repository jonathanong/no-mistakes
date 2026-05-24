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

fn settings_with_html_ids() -> Settings {
    Settings {
        html_ids: true,
        ..settings()
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
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Label
            && target.text == "Named email"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "named-email-input")
    }));
    assert_role(&targets, "Subscribe label", "checkbox");
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::AccessibleName
            && target.text == "Plan label"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "plan-input")
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
    assert!(targets.iter().any(|target| {
        target.text == "Hello World"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "joined-text")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Again"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "joined-text")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Member"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "member-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Nested"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "nested-member-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "HTML id"
            && target.role.as_deref() == Some("button")
            && target.kind == AppTextKind::VisibleText
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Descendant save"
            && target.role.as_deref() == Some("button")
            && target.kind == AppTextKind::AccessibleName
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "descendant-button")
    }));
    assert!(!targets.iter().any(|target| {
        target.text == "Descendant save"
            && target.kind == AppTextKind::VisibleText
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "descendant-button")
    }));
    assert!(!targets.iter().any(|target| {
        target.text == "Container child"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "container-target")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Submit form"
            && target.role.as_deref() == Some("button")
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "submit-input")
    }));
    assert_role(&targets, "Explicit role", "button");
    assert_role(&targets, "Docs", "link");
    assert_role(&targets, "Dynamic docs", "link");
    assert_role(&targets, "Heading", "heading");
    assert_role(&targets, "Hero image", "img");
    assert_role(&targets, "Subscribe", "checkbox");
    assert_role(&targets, "Pick one", "radio");
    assert_role(&targets, "Volume", "slider");
    assert_role(&targets, "Search site", "searchbox");
    assert_role(&targets, "Count", "spinbutton");
    assert_role(&targets, "Country", "combobox");
    assert_role(&targets, "Tags", "listbox");
    assert_role(&targets, "Regions", "listbox");
    assert_role(&targets, "Message", "textbox");
    assert!(targets.iter().any(|target| {
        target.text == "Hidden token"
            && target.role.is_none()
            && target.kind == AppTextKind::AccessibleName
    }));
    assert!(targets.iter().any(|target| target.text == "Before"));
}

#[test]
fn normalize_locator_text_rejects_blank_text() {
    assert_eq!(normalize_locator_text(" \n\t "), None);
}

#[test]
fn extracts_html_id_refs_when_enabled() {
    let path = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "app-text-targets.tsx",
    ]);
    let root = path.parent().unwrap();
    let source = std::fs::read_to_string(&path).expect("fixture should read");
    let targets = extract_app_text_targets(root, &path, &source, &settings_with_html_ids())
        .expect("fixture parses");

    assert!(targets.iter().any(|target| {
        target.text == "HTML id"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.attribute == "id" && selector.value == "html-id-button")
    }));
}

fn assert_role(targets: &[AppTextTarget], text: &str, role: &str) {
    assert!(targets
        .iter()
        .any(|target| target.text == text && target.role.as_deref() == Some(role)));
}
