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
        route_coverage_sources: Vec::new(),
        route_coverage_vitest: Default::default(),
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
        warnings: Vec::new(),
        failures: Vec::new(),
        snapshot: Arc::new(VisiblePathSnapshot::from_paths(Path::new("/repo"), &[])),
        selections: vec![PreparedRuleSelection {
            selection: super::super::selection::RuleSelection {
                playwright_project: project.map(str::to_string),
                app: app.map(str::to_string),
                ..Default::default()
            },
            settings: Prerequisite::Ready(settings(project)),
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

#[test]
fn partial_preparation_retains_valid_config_and_failure_location() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/playwright/partial-config-failure/.no-mistakes.yml"
    ))
    .unwrap();
    let snapshot = Arc::new(VisiblePathSnapshot::from_paths(
        &root,
        &[
            root.join("playwright.good.config.ts"),
            root.join("playwright.bad.config.ts"),
            root.join("tests/good.spec.ts"),
        ],
    ));
    let prepared = prepare_with_settings(
        &root,
        &config,
        snapshot.clone(),
        Default::default(),
        Default::default(),
        PreparationOptions {
            partial: true,
            ..Default::default()
        },
        &mut |project, _, _| {
            let mut settings = settings(project.as_deref());
            settings.playwright_configs = vec![
                root.join("playwright.good.config.ts"),
                root.join("playwright.bad.config.ts"),
            ];
            Ok(settings)
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(prepared.selections.len(), 1);
    assert!(matches!(
        prepared.selections[0].settings,
        Prerequisite::Failed(_)
    ));
    assert_eq!(prepared.failures().len(), 1);
    assert_eq!(prepared.failures()[0].property.as_deref(), Some("testDir"));
    assert!(prepared.failures()[0].message.contains("project broken"));
    assert!(prepared.failures()[0]
        .file
        .as_ref()
        .unwrap()
        .ends_with("playwright.bad.config.ts"));
    assert!(
        super::super::prepared_entrypoints::prepare_from_snapshot_with_catalog(
            &root,
            None,
            &config,
            snapshot,
            Default::default(),
            Arc::new(crate::codebase::ts_resolver::TsConfigCatalog::forced(
                &root,
                Default::default(),
                None
            )),
            Default::default(),
        )
        .is_err()
    );
}

#[test]
fn partial_project_prerequisites_keep_unrelated_selection_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    for (filename, complete) in [
        ("playwright.projects.config.ts", true),
        ("playwright.shared.config.ts", false),
    ] {
        let path = root.join(filename);
        let snapshot = Arc::new(VisiblePathSnapshot::from_paths(
            &root,
            &[path.clone(), root.join("tests/good.spec.ts")],
        ));
        let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
            "../../../../../../fixtures/playwright/partial-config-failure/.no-mistakes.yml"
        ))
        .unwrap();
        config.rules[0].tests.playwright = vec!["good".into()];
        let prepared = prepare_with_settings(
            &root,
            &config,
            snapshot,
            Default::default(),
            Default::default(),
            PreparationOptions {
                partial: true,
                ..Default::default()
            },
            &mut |project, _, _| {
                let mut settings = settings(project.as_deref());
                settings.playwright_configs = vec![path.clone()];
                Ok(settings)
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(prepared.failures().is_empty(), complete, "{filename}");
        assert_eq!(
            matches!(prepared.selections[0].settings, Prerequisite::Ready(_)),
            complete,
            "{filename}"
        );
    }
}

#[test]
fn selected_named_config_ignores_other_named_config_failure() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    let paths = vec![
        root.join("playwright.good.config.ts"),
        root.join("playwright.bad.config.ts"),
    ];
    let snapshot = Arc::new(VisiblePathSnapshot::from_paths(&root, &paths));
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/playwright/partial-config-failure/.no-mistakes.yml"
    ))
    .unwrap();
    config.rules[0].tests.playwright = vec!["good".into()];
    let prepared = prepare_with_settings(
        &root,
        &config,
        snapshot,
        Default::default(),
        Default::default(),
        PreparationOptions {
            partial: true,
            ..Default::default()
        },
        &mut |project, _, _| {
            let mut settings = settings(project.as_deref());
            settings.playwright_configs = paths.clone();
            Ok(settings)
        },
    )
    .unwrap()
    .unwrap();
    assert!(prepared.failures().is_empty());
    assert!(matches!(
        prepared.selections[0].settings,
        Prerequisite::Ready(_)
    ));
}

#[test]
fn sole_failed_project_is_not_a_ready_empty_selection() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    let path = root.join("playwright.bad.config.ts");
    let snapshot = Arc::new(VisiblePathSnapshot::from_paths(
        &root,
        std::slice::from_ref(&path),
    ));
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/playwright/partial-config-failure/.no-mistakes.yml"
    ))
    .unwrap();
    let prepared = prepare_with_settings(
        &root,
        &config,
        snapshot,
        Default::default(),
        Default::default(),
        PreparationOptions {
            partial: true,
            ..Default::default()
        },
        &mut |project, _, _| {
            let mut settings = settings(project.as_deref());
            settings.playwright_configs = vec![path.clone()];
            Ok(settings)
        },
    )
    .unwrap()
    .unwrap();
    assert!(matches!(
        prepared.selections[0].settings,
        Prerequisite::Failed(_)
    ));
}

#[test]
fn settings_failure_is_owned_by_partial_selection_and_legacy_error_is_preserved() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    let config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/playwright/partial-config-failure/.no-mistakes.yml"
    ))
    .unwrap();
    for partial in [false, true] {
        let snapshot = Arc::new(VisiblePathSnapshot::from_paths(&root, &[]));
        let result = prepare_with_settings(
            &root,
            &config,
            snapshot.clone(),
            Default::default(),
            Default::default(),
            PreparationOptions {
                partial,
                ..Default::default()
            },
            &mut |_, _, _| anyhow::bail!("selected project settings unavailable"),
        );
        if !partial {
            assert!(result
                .err()
                .unwrap()
                .to_string()
                .contains("selected project settings unavailable"));
            continue;
        }
        let prepared = result.unwrap().unwrap();
        assert!(Arc::ptr_eq(&prepared.snapshot, &snapshot));
        assert!(!prepared.failures().is_empty());
        assert!(prepared.failures().iter().all(|failure| failure
            .message
            .contains("selected project settings unavailable")));
        assert!(matches!(
            prepared.selections[0].settings,
            Prerequisite::Failed(_)
        ));
        assert!(prepared.report_view(None, None, true).is_none());
    }
}

#[test]
fn unknown_project_fact_selection_is_incomplete_only_in_partial_mode() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/playwright/partial-config-failure");
    let path = root.join("playwright.good.config.ts");
    let mut config: NoMistakesConfig = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/playwright/partial-config-failure/.no-mistakes.yml"
    ))
    .unwrap();
    config.rules[0].tests.playwright = vec!["missing-project".into()];
    for partial in [false, true] {
        let result = prepare_with_settings(
            &root,
            &config,
            Arc::new(VisiblePathSnapshot::from_paths(
                &root,
                std::slice::from_ref(&path),
            )),
            Default::default(),
            Default::default(),
            PreparationOptions {
                partial,
                ..Default::default()
            },
            &mut |project, _, _| {
                let mut settings = settings(project.as_deref());
                settings.playwright_configs = vec![path.clone()];
                Ok(settings)
            },
        );
        if !partial {
            assert!(result
                .err()
                .unwrap()
                .to_string()
                .contains("missing-project"));
            continue;
        }
        let prepared = result.unwrap().unwrap();
        assert!(!prepared.failures().is_empty());
        assert!(prepared
            .failures()
            .iter()
            .all(|failure| failure.message.contains("missing-project")));
        assert!(matches!(
            prepared.selections[0].settings,
            Prerequisite::Failed(_)
        ));
        assert!(prepared
            .report_view(Some("missing-project"), None, false)
            .is_none());
    }
}
