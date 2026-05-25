use super::*;
use std::path::PathBuf;
use std::sync::Arc;

fn target(kind: AppTextKind, text: &str, hidden: bool) -> AppTextTarget {
    AppTextTarget {
        file: PathBuf::from("app.tsx"),
        app_file: Arc::new("app.tsx".to_string()),
        kind,
        role: Some("button".to_string()),
        text: text.to_string(),
        hidden,
        selector_refs: Vec::new(),
    }
}

#[test]
fn text_match_handles_exact_and_case_insensitive_substrings() {
    let exact = TextMatch::new("Save", true);
    assert!(exact.matches("Save"));
    assert!(!exact.matches("save"));

    let fuzzy = TextMatch::new("save", false);
    assert!(fuzzy.matches("Click SAVE now"));
    assert!(!fuzzy.matches("Cancel"));
}

#[test]
fn role_matching_respects_hidden_targets() {
    let accessible = target(AppTextKind::AccessibleName, "Save", false);
    let hidden = target(AppTextKind::AccessibleName, "Save", true);
    let text = TextMatch::new("Save", true);

    assert!(text_target_matches(
        &accessible,
        &LocatorKind::Role,
        Some("button"),
        &text,
        false
    ));
    assert!(!text_target_matches(
        &hidden,
        &LocatorKind::Role,
        Some("button"),
        &text,
        false
    ));
    assert!(text_target_matches(
        &hidden,
        &LocatorKind::Role,
        Some("button"),
        &text,
        true
    ));
}

#[test]
fn role_matching_ignores_visible_text_names() {
    let visible = target(AppTextKind::VisibleText, "Save", false);
    let text = TextMatch::new("Save", true);

    assert!(!text_target_matches(
        &visible,
        &LocatorKind::Role,
        Some("button"),
        &text,
        false
    ));
}
