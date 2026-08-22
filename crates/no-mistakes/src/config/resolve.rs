use anyhow::Result;
use serde::Serialize;
use std::path::Path;

use super::v2::schema::{NamedFullSuiteTrigger, NoMistakesConfig};
use super::v2::{frontend_apps, load_v2_config_with_path, FrontendApp};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedConfig {
    pub config_path: Option<String>,
    pub frontend_apps: Vec<ResolvedFrontendApp>,
    pub playwright: ResolvedPlaywright,
    pub vitest_full_suite_triggers: Vec<ResolvedTrigger>,
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTrigger {
    pub name: String,
    pub paths: Vec<String>,
    pub targets: Vec<String>,
    pub source: &'static str,
}

pub fn resolve_config(root: &Path, config: Option<&Path>) -> Result<ResolvedConfig> {
    let (config, config_path) = load_v2_config_with_path(root, config)?;
    let visible = crate::codebase::ts_source::discover_visible_paths(root);
    let apps = frontend_apps(root, &config, &visible)?;
    Ok(ResolvedConfig {
        config_path: config_path.map(|path| display_rel(root, &path)),
        playwright: resolved_playwright(&config, &apps),
        frontend_apps: apps.into_iter().map(resolved_app).collect(),
        vitest_full_suite_triggers: resolved_triggers(&config),
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
            .map(|(name, binding)| resolved_playwright_app(name, binding, apps))
            .collect(),
    }
}

fn resolved_playwright_app(
    name: &str,
    binding: &crate::config::v2::schema::PlaywrightAppBinding,
    apps: &[FrontendApp],
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
            .or_else(|| inherited.map(|app| app.route_root.clone())),
        selector_roots: if binding.selector_roots.is_empty() {
            inherited
                .map(|app| app.selector_roots.clone())
                .unwrap_or_default()
        } else {
            binding.selector_roots.clone()
        },
    }
}

fn resolved_triggers(config: &NoMistakesConfig) -> Vec<ResolvedTrigger> {
    let mut triggers = config
        .test_plan
        .vitest
        .full_suite_triggers
        .triggers
        .iter()
        .map(named_trigger)
        .collect::<Vec<_>>();
    for (name, dependency) in &config.test_plan.vitest.full_suite_triggers.projects {
        if let Some(trigger) = project_trigger(name, dependency) {
            triggers.push(trigger);
        }
    }
    triggers
}

fn named_trigger(trigger: &NamedFullSuiteTrigger) -> ResolvedTrigger {
    ResolvedTrigger {
        name: trigger.name.clone(),
        paths: trigger.paths.clone(),
        targets: trigger.targets.clone(),
        source: "triggers",
    }
}

fn project_trigger(
    name: &str,
    dependency: &crate::config::v2::schema::TestPlanProjectDependency,
) -> Option<ResolvedTrigger> {
    use crate::config::v2::schema::TestPlanProjectDependency;
    Some(match dependency {
        TestPlanProjectDependency::All(false) => return None,
        TestPlanProjectDependency::All(true) => ResolvedTrigger {
            name: name.to_string(),
            paths: Vec::new(),
            targets: Vec::new(),
            source: "projects",
        },
        TestPlanProjectDependency::Patterns(paths) => ResolvedTrigger {
            name: name.to_string(),
            paths: paths.clone(),
            targets: Vec::new(),
            source: "projects",
        },
        TestPlanProjectDependency::Targeted(targeted) => ResolvedTrigger {
            name: name.to_string(),
            paths: targeted.paths.clone(),
            targets: targeted.targets.clone(),
            source: "projects",
        },
    })
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
