use super::*;

#[test]
fn labelledby_targets_do_not_require_target_ids() {
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
        target.kind == AppTextKind::AccessibleName
            && target.text == "No id label"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "labelled-no-id")
    }));
    assert!(targets.iter().any(|target| {
        target.kind == AppTextKind::Label
            && target.text == "No id label"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "labelled-no-id")
    }));
}

#[test]
fn labelledby_suppresses_descendant_accessible_names() {
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
        target.kind == AppTextKind::AccessibleName
            && target.text == "Override name"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "labelledby-precedence")
    }));
    assert!(!targets.iter().any(|target| {
        target.kind == AppTextKind::AccessibleName
            && target.text == "Visible should not name"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "labelledby-precedence")
    }));
    assert!(!targets.iter().any(|target| {
        target.kind == AppTextKind::AccessibleName
            && target.text == "Dynamic visible should not name"
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == "dynamic-labelledby")
    }));
}
