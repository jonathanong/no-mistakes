use super::*;

fn fixture_targets() -> Vec<AppTextTarget> {
    let path = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "app-text-targets.tsx",
    ]);
    let root = path.parent().unwrap();
    let source = std::fs::read_to_string(&path).expect("fixture should read");
    extract_app_text_targets(root, &path, &source, &settings()).expect("fixture parses")
}

#[test]
fn input_button_values_emit_accessible_names() {
    let targets = fixture_targets();

    assert!(targets.iter().any(|target| {
        target.text == "Submit form"
            && target.kind == AppTextKind::AccessibleName
            && target.role.as_deref() == Some("button")
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "submit-input")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Case submit form"
            && target.kind == AppTextKind::AccessibleName
            && target.role.as_deref() == Some("button")
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "submit-case-input")
    }));
}

#[test]
fn alt_names_only_apply_to_supported_elements() {
    let targets = fixture_targets();

    assert!(!targets.iter().any(|target| {
        target.text == "Ignored alt"
            && target.kind == AppTextKind::AccessibleName
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "button-alt")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "Image submit" && target.role.as_deref() == Some("button")
    }));
}

#[test]
fn hidden_inputs_are_not_label_targets() {
    let targets = fixture_targets();

    assert!(!targets
        .iter()
        .any(|target| target.text == "Hidden token" && target.kind == AppTextKind::Label));
    assert!(!targets
        .iter()
        .any(|target| target.text == "Hidden label" && target.kind == AppTextKind::Label));
}

#[test]
fn non_null_href_expressions_keep_link_role() {
    let targets = fixture_targets();

    assert!(targets
        .iter()
        .any(|target| target.text == "Zero link" && target.role.as_deref() == Some("link")));
}

#[test]
fn hidden_descendant_text_is_not_accessible_name_text() {
    let targets = fixture_targets();

    assert!(targets.iter().any(|target| {
        target.text == "Shown descendant"
            && target.kind == AppTextKind::AccessibleName
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-descendant-button")
    }));
    assert!(!targets.iter().any(|target| {
        target.text == "Decorative hidden"
            && target.kind == AppTextKind::AccessibleName
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-descendant-button")
    }));
}

#[test]
fn ts_wrapped_jsx_attrs_drive_roles_and_hidden_state() {
    let targets = fixture_targets();

    assert!(targets.iter().any(|target| {
        target.text == "TS hidden action"
            && target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "hidden-ts-button")
    }));
    assert!(targets.iter().any(|target| {
        target.text == "TS aria shown action"
            && !target.hidden
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "aria-hidden-ts-button")
    }));
    assert!(targets
        .iter()
        .any(|target| target.text == "TS regions" && target.role.as_deref() == Some("listbox")));
}
