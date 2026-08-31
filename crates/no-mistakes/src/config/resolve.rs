use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use super::v2::schema::{
    NoMistakesConfig, PlaywrightAppBinding, PlaywrightTestConfig, RewriteRule,
};
use super::v2::{frontend_apps, load_v2_config_with_path, FrontendApp};

#[path = "resolve/triggers.rs"]
mod triggers;
use triggers::{resolved_framework_triggers, resolved_vitest_triggers};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedConfig {
    pub config_path: Option<String>,
    pub frontend_apps: Vec<ResolvedFrontendApp>,
    pub playwright: ResolvedPlaywright,
    pub vitest_full_suite_triggers: Vec<ResolvedTrigger>,
    pub full_suite_triggers: Vec<ResolvedFrameworkTriggers>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFrontendApp {
    pub project: Option<String>,
    pub root: String,
    pub route_root: String,
    pub selector_roots: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPlaywright {
    pub coverage_routes: bool,
    pub coverage_selectors: bool,
    pub frontend_root: Option<String>,
    pub selector_roots: Vec<String>,
    pub apps: Vec<ResolvedPlaywrightApp>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedPlaywrightApp {
    pub playwright_project: String,
    pub project: Option<String>,
    pub frontend_root: Option<String>,
    pub selector_roots: Vec<String>,
    pub rewrites: Vec<RewriteRule>,
    pub ignore_routes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTrigger {
    pub name: String,
    pub paths: Vec<String>,
    pub targets: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_changed_tests: Option<bool>,
    pub source: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFrameworkTriggers {
    pub framework: &'static str,
    pub triggers: Vec<ResolvedTrigger>,
}

pub fn resolve_config(root: &Path, config: Option<&Path>) -> Result<ResolvedConfig> {
    let (config, config_path) = load_v2_config_with_path(root, config)?;
    let visible = crate::codebase::ts_source::discover_visible_paths(root);
    let apps = frontend_apps(root, &config, &visible)?;
    Ok(ResolvedConfig {
        config_path: config_path.map(|path| display_rel(root, &path)),
        playwright: resolved_playwright(&config, &apps),
        frontend_apps: apps.into_iter().map(resolved_app).collect(),
        vitest_full_suite_triggers: resolved_vitest_triggers(&config),
        full_suite_triggers: resolved_framework_triggers(&config),
    })
}

fn resolved_app(app: FrontendApp) -> ResolvedFrontendApp {
    ResolvedFrontendApp {
        project: app.project,
        root: app.root,
        route_root: app.route_root,
        selector_roots: app.selector_roots,
    }
}

fn resolved_playwright(config: &NoMistakesConfig, apps: &[FrontendApp]) -> ResolvedPlaywright {
    let playwright = &config.tests.playwright;
    ResolvedPlaywright {
        coverage_routes: playwright.coverage.routes,
        coverage_selectors: playwright.coverage.selectors,
        frontend_root: playwright.frontend_root.clone(),
        selector_roots: playwright.selector_roots.clone(),
        apps: playwright
            .apps
            .iter()
            .map(|(name, binding)| resolved_playwright_app(name, binding, apps, playwright))
            .collect(),
    }
}

fn resolved_playwright_app(
    name: &str,
    binding: &PlaywrightAppBinding,
    apps: &[FrontendApp],
    playwright: &PlaywrightTestConfig,
) -> ResolvedPlaywrightApp {
    let inherited = binding.project.as_ref().and_then(|project| {
        apps.iter()
            .find(|app| app.project.as_deref() == Some(project.as_str()))
    });
    ResolvedPlaywrightApp {
        playwright_project: name.to_string(),
        project: binding.project.clone(),
        frontend_root: binding
            .frontend_root
            .clone()
            .or_else(|| playwright.frontend_root.clone())
            .or_else(|| inherited.map(|app| app.route_root.clone())),
        selector_roots: if !binding.selector_roots.is_empty() {
            binding.selector_roots.clone()
        } else if !playwright.selector_roots.is_empty() {
            playwright.selector_roots.clone()
        } else {
            inherited
                .map(|app| app.selector_roots.clone())
                .unwrap_or_default()
        },
        rewrites: if binding.rewrites.is_empty() {
            inherited
                .map(|app| app.rewrites.clone())
                .unwrap_or_default()
        } else {
            binding.rewrites.clone()
        },
        ignore_routes: binding
            .ignore_routes
            .clone()
            .or_else(|| playwright.ignore_routes.clone())
            .unwrap_or_default(),
    }
}

fn display_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
#[path = "resolve/tests.rs"]
mod tests;
