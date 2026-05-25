use super::*;

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
