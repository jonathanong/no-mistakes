use super::*;

fn targets() -> Vec<AppTextTarget> {
    let path = crate::playwright::test_support::fixture_path(&[
        "ast-snippets",
        "selectors",
        "app-text-targets.tsx",
    ]);
    let root = path.parent().unwrap();
    let source = std::fs::read_to_string(&path).expect("fixture should read");
    extract_app_text_targets(root, &path, &source, &settings()).expect("fixture parses")
}

fn has_target(
    targets: &[AppTextTarget],
    text: &str,
    kind: AppTextKind,
    selector_value: &str,
) -> bool {
    targets.iter().any(|target| {
        target.text == text
            && target.kind == kind
            && target
                .selector_refs
                .iter()
                .any(|selector| selector.value == selector_value)
    })
}

#[test]
fn jsx_attr_strings_unwrap_ts_wrappers() {
    let targets = targets();

    assert!(has_target(
        &targets,
        "TS email",
        AppTextKind::Label,
        "ts-email-input"
    ));
    assert!(has_target(
        &targets,
        "Wrapped close",
        AppTextKind::AccessibleName,
        "aria-label-ts"
    ));
}

#[test]
fn aria_names_take_precedence_over_fallback_names() {
    let targets = targets();

    assert!(has_target(
        &targets,
        "First",
        AppTextKind::AccessibleName,
        "aria-labelledby-precedence"
    ));
    assert!(!has_target(
        &targets,
        "Ignored label",
        AppTextKind::AccessibleName,
        "aria-labelledby-precedence"
    ));
    assert!(!has_target(
        &targets,
        "Ignored title",
        AppTextKind::AccessibleName,
        "aria-labelledby-precedence"
    ));
    assert!(!has_target(
        &targets,
        "Ignored empty label title",
        AppTextKind::AccessibleName,
        "empty-aria-label-title"
    ));
}

#[test]
fn submit_value_accessible_name_yields_to_aria_name() {
    let targets = targets();

    assert!(has_target(
        &targets,
        "Submit aria",
        AppTextKind::AccessibleName,
        "submit-aria-input"
    ));
    assert!(!has_target(
        &targets,
        "Submit ignored",
        AppTextKind::AccessibleName,
        "submit-aria-input"
    ));
}
