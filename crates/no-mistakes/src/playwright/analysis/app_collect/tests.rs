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

#[test]
fn app_selector_and_text_collectors_read_through_snapshot_source_store() {
    let app_collect = include_str!("../app_collect.rs");
    let app_text = include_str!("../app_text.rs");
    let cross_file = include_str!("../../selectors/dynamic_values/cross_file.rs");
    assert!(
        !app_collect.contains("std::fs::read_to_string"),
        "selector scans must reuse the snapshot SourceStore"
    );
    assert!(app_collect.contains("read_snapshot_source"));
    assert!(
        !app_text.contains("std::fs::read_to_string"),
        "text scans must reuse the snapshot SourceStore"
    );
    assert!(app_text.contains("read_snapshot_source"));
    assert!(
        !cross_file.contains("std::fs::read_to_string"),
        "imported selector sources must reuse the snapshot SourceStore"
    );
    let extract_app = include_str!("../../selectors/extract_app.rs");
    assert!(
        !extract_app.contains("std::fs::read_to_string"),
        "standalone collect_app_selectors must reuse SourceStore"
    );
    assert!(extract_app.contains("read_prepared_or_open"));
}

/// Selector and text scans walk the same files. The second pass must hit the
/// snapshot store instead of opening those files again.
#[test]
fn app_text_scan_reuses_selector_scan_source_cache() {
    let root =
        crate::playwright::test_support::fixture_path(&["config-v2", "frontend-apps-inferred"]);
    let settings = root_selector_settings();
    let regexes = compile_selector_regexes(&settings.selector_attributes, &BTreeMap::new());
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let snapshot = VisiblePathSnapshot::new_observed(&root, Some(observer.clone()));
    let sources = snapshot.source_store_for(&root);

    collect_app_selector_occurrences_from_visible(&root, &settings, &regexes, &snapshot).unwrap();
    let after_selectors = sources.physical_read_count();
    let after_selector_hits = observer
        .snapshot()
        .work
        .get("source.cache_hits")
        .copied()
        .unwrap_or_default();
    assert!(
        after_selectors > 0,
        "selector scan should read app sources once"
    );

    crate::playwright::analysis::app_text::collect_app_text_targets_from_visible(
        &root, &settings, &snapshot,
    )
    .unwrap();
    assert_eq!(
        sources.physical_read_count(),
        after_selectors,
        "text scan must reuse selector-scan source cache"
    );
    let after_text_hits = observer
        .snapshot()
        .work
        .get("source.cache_hits")
        .copied()
        .unwrap_or_default();
    assert!(
        after_text_hits > after_selector_hits,
        "text scan must record SourceStore cache hits: {after_selector_hits} -> {after_text_hits}"
    );
}
