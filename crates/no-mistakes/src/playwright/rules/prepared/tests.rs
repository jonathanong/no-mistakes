use super::*;
use std::collections::BTreeMap;

fn settings(project: Option<&str>) -> config::Settings {
    config::Settings {
        frontend_root: "app".to_string(),
        playwright_configs: vec![],
        project: project.map(str::to_string),
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_wrappers: vec![],
        selector_attributes: vec![],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    }
}

fn prepared(project: Option<&str>, app: Option<&str>) -> PreparedPlaywrightRules {
    PreparedPlaywrightRules {
        snapshot: Arc::new(VisiblePathSnapshot::from_paths(Path::new("/repo"), &[])),
        selections: vec![PreparedRuleSelection {
            selection: super::super::selection::RuleSelection {
                playwright_project: project.map(str::to_string),
                app: app.map(str::to_string),
                ..Default::default()
            },
            settings: settings(project),
        }],
        fact_plan: PlaywrightFactPlan::default(),
    }
}

/// A single configured `type: nextjs` project auto-resolves the sole
/// Playwright selection's `app` to that project's name (see
/// `resolve_selection_app`'s sole-app branch). A `report_view` caller with
/// no opinion about which app it wants — `app: None`, the common case since
/// most N-API/CLI requests never set the `app` option — must still hit that
/// prepared selection instead of missing it and silently falling back to a
/// slower, non-shared standalone resolution.
#[test]
fn report_view_matches_on_project_when_caller_has_no_app_preference() {
    let prepared = prepared(Some("main"), Some("web"));

    assert!(prepared.report_view(Some("main"), None, false).is_some());
}

/// A caller that *does* name an app must still get an exact match — this is
/// not a green light to ignore `app` entirely, only to treat its absence as
/// "no preference".
#[test]
fn report_view_requires_an_exact_match_when_the_caller_names_an_app() {
    let prepared = prepared(Some("main"), Some("web"));

    assert!(prepared
        .report_view(Some("main"), Some("web"), false)
        .is_some());
    assert!(prepared
        .report_view(Some("main"), Some("other"), false)
        .is_none());
}
