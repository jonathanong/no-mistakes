use super::{helpers, settings_from_defaults};
use crate::config::v2::schema::NoMistakesConfig;
use crate::config::v2::ConfigView;
use crate::playwright::config::Settings;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) fn settings_from_v2(
    root: &Path,
    config: &NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    let view = ConfigView::new(config);
    let playwright = &config.tests.playwright;
    let root_paths = visible_paths.paths_for(root);
    let frontend_root = playwright
        .frontend_root
        .clone()
        .unwrap_or_else(|| default_frontend_root(root, view.nextjs_root(), &root_paths));
    let playwright_configs =
        helpers::playwright_configs_from_v2(root, &view, cli_playwright_configs, &root_paths)?;
    let selector_attributes = if view.test_id_attributes().is_empty() {
        helpers::default_selector_attributes()
    } else {
        view.test_id_attributes().to_vec()
    };
    let selector_roots = if view.selector_roots().is_empty() {
        vec![frontend_root.clone()]
    } else {
        view.selector_roots().to_vec()
    };
    Ok(Settings {
        frontend_root,
        playwright_configs,
        project: cli_project,
        test_include: playwright.test_include.clone(),
        test_exclude: playwright.test_exclude.clone(),
        ignore_routes: playwright.ignore_routes.clone().unwrap_or_default(),
        rewrites: view.nextjs_rewrites().to_vec(),
        navigation_helpers: playwright.navigation_helpers.clone(),
        selector_attributes,
        test_id_attribute_override: playwright.test_id_attribute.clone(),
        component_selector_attributes: playwright.selectors.component_test_ids.clone(),
        html_ids: playwright.selectors.html_ids,
        selector_roots,
        selector_include: playwright.selector_include.clone(),
        selector_exclude: playwright.selector_exclude.clone(),
    })
}

pub(super) fn settings_from_loaded_v2(
    root: &Path,
    config: &NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    if helpers::has_v2_playwright_settings(config) {
        settings_from_v2(
            root,
            config,
            cli_playwright_configs,
            cli_project,
            visible_paths,
        )
    } else {
        settings_from_defaults(root, cli_playwright_configs, cli_project, visible_paths)
    }
}

fn default_frontend_root(root: &Path, nextjs_root: &str, visible_paths: &[PathBuf]) -> String {
    let app_root = Path::new(nextjs_root).join("app");
    let absolute_app_root = crate::codebase::ts_resolver::normalize_path(&root.join(&app_root));
    if visible_paths.iter().any(|path| {
        crate::codebase::ts_resolver::normalize_path(path).starts_with(&absolute_app_root)
    }) {
        app_root.to_string_lossy().into_owned()
    } else {
        nextjs_root.to_string()
    }
}
