use super::{helpers, settings_from_defaults};
use crate::config::v2::schema::{NoMistakesConfig, PlaywrightAppBinding};
use crate::config::v2::{ConfigView, FrontendApp};
use crate::playwright::config::Settings;
use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Build [`Settings`] for one Playwright project, resolving which frontend
/// app it exercises.
///
/// `app` is the `.no-mistakes.yml` `projects:` key the caller has already
/// bound this Playwright project to (for example
/// [`crate::playwright::rules::selection::RuleSelection::app`], resolved from
/// a rule's `projects:` list) — pass `None` when the caller has no such
/// binding (e.g. the standalone `playwright check` CLI, which has no
/// `rules:` context). Either way, `tests.playwright.apps.<cli_project>.project`
/// takes precedence when set.
pub(super) fn settings_from_v2(
    root: &Path,
    config: &NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    app: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    let view = ConfigView::new(config);
    let playwright = &config.tests.playwright;
    let root_paths = visible_paths.paths_for(root);
    let binding = cli_project
        .as_deref()
        .and_then(|project| playwright.apps.get(project));

    let frontend_root_explicit = binding
        .and_then(|binding| binding.frontend_root.clone())
        .or_else(|| playwright.frontend_root.clone());
    let selector_roots_explicit = binding
        .map(|binding| binding.selector_roots.clone())
        .filter(|roots| !roots.is_empty())
        .or_else(|| {
            let roots = view.selector_roots();
            (!roots.is_empty()).then(|| roots.to_vec())
        });
    let rewrites_explicit = binding
        .map(|binding| binding.rewrites.clone())
        .filter(|rewrites| !rewrites.is_empty());

    // Only resolve which frontend app is in play when at least one of the
    // three app-derived fields isn't already fully explicit. A Playwright
    // project that overrides all three needs no app resolution at all, so a
    // genuinely ambiguous `projects:` set elsewhere in the config does not
    // fail this project's settings.
    let needs_resolved_app = frontend_root_explicit.is_none()
        || selector_roots_explicit.is_none()
        || rewrites_explicit.is_none();
    let resolved_app = if needs_resolved_app {
        let apps = crate::config::v2::frontend_apps(root, config, &root_paths)?;
        resolve_frontend_app(binding, app.as_deref(), apps)?
    } else {
        None
    };

    let frontend_root = frontend_root_explicit
        .or_else(|| resolved_app.as_ref().map(|app| app.route_root.clone()))
        .unwrap_or_else(|| default_frontend_root(root, "app", &root_paths));
    let playwright_configs =
        helpers::playwright_configs_from_v2(root, &view, cli_playwright_configs, visible_paths)?;
    let selector_attributes = if view.test_id_attributes().is_empty() {
        helpers::default_selector_attributes()
    } else {
        view.test_id_attributes().to_vec()
    };
    let selector_roots = selector_roots_explicit
        .or_else(|| resolved_app.as_ref().map(|app| app.selector_roots.clone()))
        .unwrap_or_else(|| vec![frontend_root.clone()]);
    let rewrites = rewrites_explicit
        .or_else(|| resolved_app.as_ref().map(|app| app.rewrites.clone()))
        .unwrap_or_default();
    let ignore_routes = binding
        .and_then(|binding| binding.ignore_routes.clone())
        .or_else(|| playwright.ignore_routes.clone())
        .unwrap_or_default();

    Ok(Settings {
        frontend_root,
        playwright_configs,
        project: cli_project,
        test_include: playwright.test_include.clone(),
        test_exclude: playwright.test_exclude.clone(),
        ignore_routes,
        rewrites,
        navigation_helpers: playwright.navigation_helpers.clone(),
        selector_wrappers: playwright.selectors.wrappers.clone(),
        selector_attributes,
        test_id_attribute_override: playwright.test_id_attribute.clone(),
        component_selector_attributes: playwright.selectors.component_test_ids.clone(),
        html_ids: playwright.selectors.html_ids,
        selector_roots,
        selector_include: playwright.selector_include.clone(),
        selector_exclude: playwright.selector_exclude.clone(),
    })
}

/// Resolve which [`FrontendApp`] this Playwright project exercises.
///
/// - `Ok(Some(app))`: a specific app was named (via `binding.project` or the
///   caller-supplied `fallback_app`) or there is exactly one configured app.
/// - `Ok(None)`: there are no configured/inferred frontend apps at all — not
///   an error; callers fall back to the pre-#624 bare default.
/// - `Err`: more than one app is configured and none was named — the
///   ambiguity #624 silently resolved by picking whichever project sorted
///   first.
fn resolve_frontend_app(
    binding: Option<&PlaywrightAppBinding>,
    fallback_app: Option<&str>,
    apps: Vec<FrontendApp>,
) -> Result<Option<FrontendApp>> {
    let bound_name = binding
        .and_then(|binding| binding.project.as_deref())
        .or(fallback_app);
    if let Some(name) = bound_name {
        return apps
            .into_iter()
            .find(|app| app.project.as_deref() == Some(name))
            .map(Some)
            .ok_or_else(|| {
                anyhow!(
                    "`{name}` is not a configured `type: nextjs` project (or its root could not \
                     be resolved). Checked `tests.playwright.apps.*.project` and the rule's \
                     `projects:` list."
                )
            });
    }
    match apps.len() {
        0 => Ok(None),
        1 => Ok(apps.into_iter().next()),
        _ => {
            let names: Vec<&str> = apps
                .iter()
                .filter_map(|app| app.project.as_deref())
                .collect();
            Err(anyhow!(
                "cannot resolve which frontend app to analyze: {} `type: nextjs` projects are \
                 configured ({}).\nFix one of:\n  \
                 - add `projects: [<nextjs project>]` to the rule application\n  \
                 - set `tests.playwright.apps.<playwright-project>.project`",
                apps.len(),
                names.join(", "),
            ))
        }
    }
}

pub(super) fn settings_from_loaded_v2(
    root: &Path,
    config: &NoMistakesConfig,
    cli_playwright_configs: &[PathBuf],
    cli_project: Option<String>,
    app: Option<String>,
    visible_paths: &crate::playwright::fsutil::VisiblePathSnapshot,
) -> Result<Settings> {
    if helpers::has_v2_playwright_settings(config) {
        settings_from_v2(
            root,
            config,
            cli_playwright_configs,
            cli_project,
            app,
            visible_paths,
        )
    } else {
        settings_from_defaults(root, cli_playwright_configs, cli_project, visible_paths)
    }
}

/// Legacy zero-signal fallback: no `type: nextjs` project is configured and
/// none could be inferred at all, so there is no [`FrontendApp`] to derive a
/// route root from. Retained unchanged (still only probes `<nextjs_root>/app`,
/// not `<nextjs_root>/src/app`) because this deepest fallback tier is not the
/// path #625 reported — that case always has a resolvable [`FrontendApp`],
/// which already applies the `src/app`-preferred probe.
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

#[cfg(test)]
#[path = "loaded_v2/tests.rs"]
mod tests;
