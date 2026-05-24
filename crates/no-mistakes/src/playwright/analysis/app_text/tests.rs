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
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Label
            && target.text == "Identifier email"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "identifier-email-input")
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
    assert!(targets.iter().any(|target| {
        target.text == "Template child"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "template-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Template aria"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "template-aria")
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
    assert!(targets.iter().any(|target| {
        target.text == "Hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "String hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-string-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Expression hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-expression-string-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-false-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Null shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-null-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Undefined shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-undefined-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Zero shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-zero-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "One hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-one-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Empty template shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-empty-template-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Template hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-template-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Dynamic shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-dynamic-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Aria hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "aria-hidden-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Aria shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "aria-hidden-false-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Bool hidden action"
            && target.role.as_deref() == Some("button")
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "aria-hidden-bool-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Expression string shown action"
            && target.role.as_deref() == Some("button")
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "aria-hidden-expression-string-button")
    }));
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Label
            && target.text == "Wrapped email"
            && target.role.as_deref() == Some("textbox")
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "wrapped-email-input")
    }));
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Label
            && target.text == "Fragment wrapped"
            && target.role.as_deref() == Some("textbox")
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "fragment-wrapped-input")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Descendant save"
            && target.kind == AppTextKind::VisibleText
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "descendant-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Fragment save"
            && target.kind == AppTextKind::VisibleText
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "fragment-button")
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
    assert_role(&targets, "Empty docs", "link");
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
    assert_role(&targets, "Single tag", "combobox");
    assert_role(&targets, "Regions", "listbox");
    assert_role(&targets, "Numeric regions", "listbox");
    assert_role(&targets, "Message", "textbox");
    assert!(targets.iter().any(|target| {
        target.text == "Hidden token"
            && target.role.is_none()
            && target.kind == AppTextKind::AccessibleName
    }));
    assert!(targets.iter().any(|target| target.text == "Before"));
    assert!(targets
        .iter()
        .any(|target| target.text == "Boolean selector" && target.selector_refs.is_empty()));
    assert!(targets.iter().any(|target| {
        target.text == "Dynamic template selector" && target.selector_refs.is_empty()
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Lower member"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "lower-member-button")
    }));
    assert!(!targets
        .iter()
        .any(|target| target.text == "Undefined link" && target.role.as_deref() == Some("link")));
    assert!(!targets
        .iter()
        .any(|target| target.text == "Null link" && target.role.as_deref() == Some("link")));
    assert!(!targets
        .iter()
        .any(|target| target.text == "Zero link" && target.role.as_deref() == Some("link")));
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
