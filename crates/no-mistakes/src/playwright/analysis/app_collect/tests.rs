use super::*;
use crate::playwright::selectors::{compile_selector_regexes, AppSelectorValue};
use std::collections::BTreeMap;

/// Builds the [`Settings`] a `FrontendApp` inferred at the repository root
/// would produce (`root: ""`, see
/// `crate::config::v2::frontend_apps::frontend_apps`), scoped to the shared
/// `frontend-apps-inferred` fixture.
fn root_selector_settings() -> Settings {
    Settings {
        frontend_root: "src/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec!["data-pw".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec![String::new()],
        selector_include: vec![],
        selector_exclude: vec![],
    }
}

/// `selector_roots: [""]` (an app inferred at the repository root, see
/// `crate::config::v2::frontend_apps::FrontendApp::root`) must scan the whole
/// repository, not zero files. `PathBuf::join("")` is a documented no-op, but
/// this proves the end-to-end walk-and-extract path agrees, not just the
/// join.
#[test]
fn empty_selector_root_scans_the_whole_repository() {
    let root =
        crate::playwright::test_support::fixture_path(&["config-v2", "frontend-apps-inferred"]);
    let settings = root_selector_settings();
    let regexes = compile_selector_regexes(&settings.selector_attributes, &BTreeMap::new());
    let snapshot = VisiblePathSnapshot::new(&root);

    let selectors =
        collect_app_selector_occurrences_from_visible(&root, &settings, &regexes, &snapshot)
            .unwrap();

    let root_cta = selectors
        .iter()
        .find(|selector| selector.attribute == "data-pw")
        .expect("expected the root-level data-pw selector to be discovered");
    assert_eq!(
        root_cta.value,
        AppSelectorValue::Exact("root-cta".to_string())
    );
    assert!(root_cta.file.ends_with("src/app/page.tsx"), "{root_cta:?}");
}
